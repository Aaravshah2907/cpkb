//! Snippet data models, CRUD operations, and sequential ID generation.

use std::path::Path;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

use crate::config::{load_config, max_snippets};
use crate::db::tags::update_tags;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub description: String,
    pub use_case: String,
    pub tags: String,
    pub code: String,
    pub language: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnippetSummary {
    pub id: String,
    pub title: String,
    pub tags: String,
    pub language: String,
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn normalize_id_pattern(pattern: &str) -> String {
    pattern
        .trim()
        .replace("<ID_BEG_KEY>", "ID")
        .replace("<_.-@>", "-")
        .replace("<#######>", "#######")
        .replace("<######>", "######")
        .replace("<#####>", "#####")
        .replace("<####>", "####")
        .replace("<###>", "###")
        .replace("<##>", "##")
        .replace("<#>", "#")
}

fn pattern_parts(format_config: &crate::config::IdFormatConfig, app_dir: &Path) -> Result<(String, String, String), String> {
    if let Some(ref pat) = format_config.pattern {
        let norm = normalize_id_pattern(pat);
        if let Some(start) = norm.find('#') {
            let hash_count = norm[start..].chars().take_while(|&c| c == '#').count();
            let prefix = &norm[..start];
            let placeholder = &norm[start..start + hash_count];
            let suffix = &norm[start + hash_count..];
            return Ok((prefix.to_string(), placeholder.to_string(), suffix.to_string()));
        } else {
            return Err("ID format pattern must include at least one # placeholder.".to_string());
        }
    }

    let prefix = format_config.prefix.clone().unwrap_or_else(|| "CP".to_string());
    let width = match &format_config.width {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(4) as usize,
        Some(serde_json::Value::String(s)) if s != "auto" => s.parse::<usize>().unwrap_or(4),
        _ => {
            let limit = max_snippets(app_dir);
            limit.max(1).to_string().len()
        }
    };

    Ok((prefix, "#".repeat(width), String::new()))
}

/// Generate the next sequential snippet ID for a given format name.
pub fn generate_id(conn: &Connection, app_dir: &Path, format_name: Option<&str>) -> Result<String, String> {
    let config = load_config(app_dir);
    let resolved_name = format_name.unwrap_or(&config.snippets.default_id_format);
    let format_config = config
        .snippets
        .id_formats
        .get(resolved_name)
        .ok_or_else(|| {
            let available: Vec<&String> = config.snippets.id_formats.keys().collect();
            format!(
                "Unknown ID format '{}'. Available formats: {}",
                resolved_name,
                available.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            )
        })?;

    let (prefix, placeholder, suffix) = pattern_parts(format_config, app_dir)?;
    let width = placeholder.len();

    let mut stmt = conn
        .prepare("SELECT id FROM snippets")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;

    let mut existing: Vec<u32> = Vec::new();
    for id_res in rows {
        if let Ok(id_str) = id_res {
            if !id_str.starts_with(&prefix) {
                continue;
            }
            let mut num_part = &id_str[prefix.len()..];
            if !suffix.is_empty() {
                if !num_part.ends_with(&suffix) {
                    continue;
                }
                num_part = &num_part[..num_part.len() - suffix.len()];
            }
            if let Ok(n) = num_part.parse::<u32>() {
                existing.push(n);
            }
        }
    }

    let next_num = existing.into_iter().max().unwrap_or(0) + 1;
    let limit = max_snippets(app_dir);
    if next_num > limit {
        return Err(format!(
            "Maximum snippet count reached ({}). Update config.json to increase it.",
            limit
        ));
    }

    Ok(format!("{}{:0width$}{}", prefix, next_num, suffix, width = width))
}

/// Insert a new snippet with auto-generated sequential ID and return the generated ID.
pub fn add_snippet(
    conn: &mut Connection,
    app_dir: &Path,
    title: &str,
    description: &str,
    use_case: &str,
    tags: &str,
    code: &str,
    mut language: Option<&str>,
    id_format: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let config = load_config(app_dir);
    let mut resolved_id_format = id_format;

    // Handle language / id_format positional resolution
    if resolved_id_format.is_none() {
        if let Some(lang) = language {
            if config.snippets.id_formats.contains_key(lang)
                && !matches!(
                    lang,
                    "cpp" | "c" | "python" | "py" | "rust" | "rs" | "javascript" | "js"
                        | "typescript" | "ts" | "go" | "golang" | "java" | "lua"
                        | "bash" | "sh" | "markdown" | "md" | "text" | "txt" | "sql"
                        | "tex" | "latex" | "plaintex"
                )
            {
                resolved_id_format = Some(lang);
                language = None;
            }
        }
    }

    let mut final_language = language.unwrap_or_else(|| {
        if let Some(fmt) = resolved_id_format {
            if fmt.to_lowercase().contains("latex") || fmt.to_lowercase().contains("tex") {
                return "tex";
            }
        }
        if !config.snippets.code_language.is_empty() {
            &config.snippets.code_language
        } else {
            &config.default_language
        }
    });

    let snippet_id = generate_id(conn, app_dir, resolved_id_format)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    if (snippet_id.starts_with("LATEX") || snippet_id.to_lowercase().starts_with("latex"))
        && final_language == "cpp"
    {
        final_language = "tex";
    }

    let now = now_iso();
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO snippets (id, title, description, use_case, tags, code, language, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![snippet_id, title, description, use_case, tags, code, final_language, now, now],
    )?;
    update_tags(&tx, &snippet_id, tags)?;
    tx.commit()?;

    Ok(snippet_id)
}

/// Insert a snippet with an explicit ID and timestamps (used by import).
pub fn insert_snippet_with_id(
    conn: &mut Connection,
    snippet_id: &str,
    title: &str,
    description: &str,
    use_case: &str,
    tags: &str,
    code: &str,
    mut language: &str,
    created_at: Option<&str>,
    updated_at: Option<&str>,
) -> Result<(), rusqlite::Error> {
    if (snippet_id.starts_with("LATEX") || snippet_id.to_lowercase().starts_with("latex"))
        && (language == "cpp" || language.is_empty())
    {
        language = "tex";
    }
    let now = now_iso();
    let c_at = created_at.unwrap_or(&now);
    let u_at = updated_at.unwrap_or(&now);

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO snippets (id, title, description, use_case, tags, code, language, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![snippet_id, title, description, use_case, tags, code, language, c_at, u_at],
    )?;
    update_tags(&tx, snippet_id, tags)?;
    tx.commit()?;
    Ok(())
}

/// Fetch a single snippet by ID.
pub fn get_snippet(conn: &Connection, snippet_id: &str) -> Result<Option<Snippet>> {
    conn.query_row(
        "SELECT id, title, description, use_case, tags, code, language, created_at, updated_at
         FROM snippets WHERE id = ?1",
        params![snippet_id],
        |row| {
            Ok(Snippet {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                use_case: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                tags: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                code: row.get(5)?,
                language: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .optional()
}

/// Update snippet fields.
pub fn update_snippet(
    conn: &mut Connection,
    snippet_id: &str,
    title: &str,
    description: &str,
    use_case: &str,
    tags: &str,
    code: &str,
    language: &str,
) -> Result<bool, rusqlite::Error> {
    let now = now_iso();
    let tx = conn.transaction()?;
    let rows_affected = tx.execute(
        "UPDATE snippets SET title = ?1, description = ?2, use_case = ?3, tags = ?4,
         code = ?5, language = ?6, updated_at = ?7 WHERE id = ?8",
        params![title, description, use_case, tags, code, language, now, snippet_id],
    )?;
    if rows_affected > 0 {
        update_tags(&tx, snippet_id, tags)?;
    }
    tx.commit()?;
    Ok(rows_affected > 0)
}

/// Delete a snippet and its cascading relations.
pub fn delete_snippet(conn: &Connection, snippet_id: &str) -> Result<bool> {
    let rows = conn.execute("DELETE FROM snippets WHERE id = ?1", params![snippet_id])?;
    Ok(rows > 0)
}

/// Sort order field options for listing snippets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetSortField {
    Id,
    Name,
    Date,
}

impl std::str::FromStr for SnippetSortField {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "id" => Ok(SnippetSortField::Id),
            "name" | "title" => Ok(SnippetSortField::Name),
            "date" | "recent" => Ok(SnippetSortField::Date),
            other => Err(format!("Unknown sort field: '{other}'. Choose from: id, name, date")),
        }
    }
}

/// List all snippets with configurable sort order.
pub fn list_snippets_sorted(conn: &Connection, sort_field: SnippetSortField) -> Result<Vec<SnippetSummary>> {
    let order_sql = match sort_field {
        SnippetSortField::Id => "ORDER BY id ASC",
        SnippetSortField::Name => "ORDER BY title COLLATE NOCASE ASC",
        SnippetSortField::Date => "ORDER BY created_at DESC, id DESC",
    };
    let sql = format!("SELECT id, title, tags, language FROM snippets {}", order_sql);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(SnippetSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            tags: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            language: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// List all snippets sorted by creation date descending.
pub fn list_snippets(conn: &Connection) -> Result<Vec<SnippetSummary>> {
    list_snippets_sorted(conn, SnippetSortField::Date)
}

/// List the most recent snippets.
pub fn recent_snippets(conn: &Connection, limit: u32) -> Result<Vec<SnippetSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, tags, language FROM snippets ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(SnippetSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            tags: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            language: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// List ALL snippets with full data, ordered by creation date ascending (for export).
pub fn list_all_snippets(conn: &Connection) -> Result<Vec<FullSnippetExport>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, use_case, tags, code, language, created_at, updated_at
         FROM snippets ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(FullSnippetExport {
            id:          row.get(0)?,
            title:       row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            description: row.get(2)?,
            use_case:    row.get(3)?,
            tags:        row.get(4)?,
            code:        row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            language:    row.get(6)?,
            created_at:  row.get(7)?,
            updated_at:  row.get(8)?,
        })
    })?;
    let mut result = Vec::new();
    for r in rows { result.push(r?); }
    Ok(result)
}

/// Return true if a snippet with `id` exists.
pub fn snippet_exists(conn: &Connection, id: &str) -> Result<bool> {
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM snippets WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Generate a new ID without requiring `app_dir` — uses the live DB to infer
/// the default format. Falls back to `generate_id` with `None` format.
/// Exposed for the import engine which holds only a `Connection`.
pub fn generate_next_id(conn: &Connection, id_format: Option<&str>) -> anyhow::Result<String> {
    // We cannot call load_config without app_dir here; use a CP#### fallback
    // derived purely from existing IDs so we don't need the FS.
    let prefix = id_format
        .map(|f| f.to_uppercase())
        .unwrap_or_else(|| "CP".to_string());

    let mut stmt = conn.prepare("SELECT id FROM snippets")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut max_num: u32 = 0;
    for id_res in rows {
        if let Ok(id_str) = id_res {
            if id_str.starts_with(&prefix) {
                let num_part = &id_str[prefix.len()..];
                if let Ok(n) = num_part.parse::<u32>() {
                    max_num = max_num.max(n);
                }
            }
        }
    }

    let next = max_num + 1;
    let width = 4.max(next.to_string().len());
    Ok(format!("{}{:0width$}", prefix, next, width = width))
}

/// Count all snippets in the database.
pub fn count_snippets(conn: &Connection) -> Result<u64> {
    conn.query_row("SELECT COUNT(*) FROM snippets", [], |row| row.get(0))
}

/// A rich snippet record for export operations (all fields optional-aware).
#[derive(Debug, Clone)]
pub struct FullSnippetExport {
    pub id:          String,
    pub title:       String,
    pub description: Option<String>,
    pub use_case:    Option<String>,
    pub tags:        Option<String>,
    pub code:        String,
    pub language:    Option<String>,
    pub created_at:  Option<String>,
    pub updated_at:  Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::get_in_memory_conn;

    #[test]
    fn test_list_snippets_sorted_by_id_and_name() {
        let mut conn = get_in_memory_conn().unwrap();
        insert_snippet_with_id(&mut conn, "CP0002", "Zebra Algorithm", "", "", "", "", "cpp", Some("2026-01-01T00:00:00Z"), None).unwrap();
        insert_snippet_with_id(&mut conn, "CP0001", "Apple Search", "", "", "", "", "rust", Some("2026-01-02T00:00:00Z"), None).unwrap();
        insert_snippet_with_id(&mut conn, "CP0003", "Banana Sort", "", "", "", "", "python", Some("2026-01-03T00:00:00Z"), None).unwrap();

        // Sort by ID
        let by_id = list_snippets_sorted(&conn, SnippetSortField::Id).unwrap();
        assert_eq!(by_id[0].id, "CP0001");
        assert_eq!(by_id[1].id, "CP0002");
        assert_eq!(by_id[2].id, "CP0003");

        // Sort by Name
        let by_name = list_snippets_sorted(&conn, SnippetSortField::Name).unwrap();
        assert_eq!(by_name[0].title, "Apple Search");
        assert_eq!(by_name[1].title, "Banana Sort");
        assert_eq!(by_name[2].title, "Zebra Algorithm");

        // Sort by Date (descending)
        let by_date = list_snippets_sorted(&conn, SnippetSortField::Date).unwrap();
        assert_eq!(by_date[0].id, "CP0003");
        assert_eq!(by_date[1].id, "CP0001");
        assert_eq!(by_date[2].id, "CP0002");
    }

    #[test]
    fn test_snippet_sort_field_from_str() {
        use std::str::FromStr;
        assert_eq!(SnippetSortField::from_str("id").unwrap(), SnippetSortField::Id);
        assert_eq!(SnippetSortField::from_str("ID").unwrap(), SnippetSortField::Id);
        assert_eq!(SnippetSortField::from_str("name").unwrap(), SnippetSortField::Name);
        assert_eq!(SnippetSortField::from_str("title").unwrap(), SnippetSortField::Name);
        assert_eq!(SnippetSortField::from_str("date").unwrap(), SnippetSortField::Date);
        assert_eq!(SnippetSortField::from_str("recent").unwrap(), SnippetSortField::Date);
        assert!(SnippetSortField::from_str("invalid").is_err());
    }
}


