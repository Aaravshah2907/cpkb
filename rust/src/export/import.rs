//! Import engine: reads Markdown, JSON, or raw-DB exports back into CPKB.
//!
//! Mirrors the Python `_load_import_snippets` / `import_snippets` logic.

use std::path::Path;
use serde::Deserialize;
use rusqlite::Connection;

use crate::db::snippets::{generate_next_id, insert_snippet_with_id, snippet_exists};

// ─── Common snippet record used during import ──────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct ImportRecord {
    pub id:          Option<String>,
    pub title:       String,
    pub description: Option<String>,
    pub use_case:    Option<String>,
    pub tags:        Option<String>,
    pub code:        String,
    pub language:    Option<String>,
    pub created_at:  Option<String>,
    pub updated_at:  Option<String>,
}

// ─── Summary returned from `import_snippets` ──────────────────────────────

#[derive(Debug, Default)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped:  usize,
    /// Maps source_id → target_id for any row that was written.
    pub id_map:   std::collections::HashMap<String, String>,
}

// ─── Format detection ──────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum ImportFormat { Json, Markdown, Html, Db }

/// Infer format from file extension; falls back to `Markdown`.
pub fn detect_format(path: &str) -> ImportFormat {
    let lower = path.to_lowercase();
    if lower.ends_with(".json")                    { ImportFormat::Json }
    else if lower.ends_with(".md") || lower.ends_with(".markdown") { ImportFormat::Markdown }
    else if lower.ends_with(".html") || lower.ends_with(".htm")    { ImportFormat::Html }
    else if lower.ends_with(".db") || lower.ends_with(".sqlite")   { ImportFormat::Db }
    else                                           { ImportFormat::Markdown }
}

// ─── Format-specific loaders ──────────────────────────────────────────────

/// Deserialise a JSON array export (produced by `export-json`).
#[derive(Deserialize)]
struct JsonSnippet {
    id:          Option<String>,
    title:       Option<String>,
    description: Option<String>,
    use_case:    Option<String>,
    tags:        Option<String>,
    code:        Option<String>,
    language:    Option<String>,
    created_at:  Option<String>,
    updated_at:  Option<String>,
}

pub fn load_json(bytes: &[u8]) -> anyhow::Result<Vec<ImportRecord>> {
    let raw: Vec<JsonSnippet> = serde_json::from_slice(bytes)?;
    Ok(raw
        .into_iter()
        .map(|j| ImportRecord {
            id:          j.id,
            title:       j.title.unwrap_or_default(),
            description: j.description,
            use_case:    j.use_case,
            tags:        j.tags,
            code:        j.code.unwrap_or_default(),
            language:    j.language,
            created_at:  j.created_at,
            updated_at:  j.updated_at,
        })
        .collect())
}

/// Parse a Markdown export (produced by `export`).
///
/// Expected format per snippet (one block per snippet):
///
/// ```text
/// ## Title (ID)
/// **Description:** ...
/// **Use case:** ...
/// **Tags:** ...
///
/// ` ` `lang
/// ... code ...
/// ` ` `
/// ```
pub fn load_markdown(text: &str) -> Vec<ImportRecord> {
    let mut records: Vec<ImportRecord> = Vec::new();
    let mut current: Option<ImportRecord> = None;
    let mut in_code_block = false;
    let mut code_lines: Vec<&str> = Vec::new();

    for line in text.lines() {
        if line.starts_with("## ") {
            // Flush the previous record
            if let Some(mut rec) = current.take() {
                rec.code = code_lines.join("\n");
                code_lines.clear();
                records.push(rec);
            }

            // Parse: ## Title (ID)
            let rest = &line[3..];
            let (title, id) = if let Some(start) = rest.rfind('(') {
                if rest.ends_with(')') {
                    let id = rest[start + 1..rest.len() - 1].to_string();
                    let title = rest[..start].trim().to_string();
                    (title, Some(id))
                } else {
                    (rest.to_string(), None)
                }
            } else {
                (rest.to_string(), None)
            };

            current = Some(ImportRecord { id, title, ..Default::default() });
            in_code_block = false;
        } else if let Some(ref mut rec) = current {
            if in_code_block {
                if line == "```" {
                    in_code_block = false;
                } else {
                    code_lines.push(line);
                }
            } else if line.starts_with("**Description:**") {
                rec.description = Some(line["**Description:**".len()..].trim().to_string());
            } else if line.starts_with("**Use case:**") {
                rec.use_case = Some(line["**Use case:**".len()..].trim().to_string());
            } else if line.starts_with("**Tags:**") {
                rec.tags = Some(line["**Tags:**".len()..].trim().to_string());
            } else if line.starts_with("```") {
                // Extract language from opening fence
                let lang = line[3..].trim();
                if !lang.is_empty() {
                    rec.language = Some(lang.to_string());
                }
                in_code_block = true;
            }
        }
    }

    // Flush last record
    if let Some(mut rec) = current {
        rec.code = code_lines.join("\n");
        records.push(rec);
    }

    records
}

/// Read snippets directly from a raw SQLite DB export.
pub fn load_db(db_path: &Path) -> anyhow::Result<Vec<ImportRecord>> {
    let conn = rusqlite::Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, title, description, use_case, tags, code, language, created_at, updated_at FROM snippets"
    )?;
    let records = stmt.query_map([], |row| {
        Ok(ImportRecord {
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
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(records)
}

/// Minimal HTML import: strip tags and extract code blocks from `<pre>` elements.
///
/// This is a best-effort fallback; the JSON and Markdown importers are preferred.
pub fn load_html(html: &str) -> Vec<ImportRecord> {
    let mut records: Vec<ImportRecord> = Vec::new();
    // State machine: look for <section>, <h2>, <pre> blocks
    let mut title = String::new();
    let mut description = String::new();
    let mut use_case = String::new();
    let mut tags_str = String::new();
    let mut in_pre = false;
    let mut code_buf = String::new();

    for line in html.lines() {
        let trimmed = line.trim();
        if trimmed.contains("<h2>") {
            // Flush previous
            if !title.is_empty() && !code_buf.is_empty() {
                records.push(ImportRecord {
                    title: strip_html_tags(&title),
                    description: Some(strip_html_tags(&description)),
                    use_case: Some(strip_html_tags(&use_case)),
                    tags: Some(strip_html_tags(&tags_str)),
                    code: code_buf.trim().to_string(),
                    ..Default::default()
                });
            }
            title = trimmed.to_string();
            description.clear(); use_case.clear(); tags_str.clear();
            code_buf.clear(); in_pre = false;
        } else if trimmed.contains("Description:") {
            description = trimmed.to_string();
        } else if trimmed.contains("Use case:") {
            use_case = trimmed.to_string();
        } else if trimmed.contains("Tags:") {
            tags_str = trimmed.to_string();
        } else if trimmed.starts_with("<pre>") {
            in_pre = true;
            let inner = trimmed.trim_start_matches("<pre>").trim_end_matches("</pre>");
            code_buf.push_str(inner);
        } else if trimmed.ends_with("</pre>") {
            in_pre = false;
        } else if in_pre {
            code_buf.push('\n');
            code_buf.push_str(trimmed);
        }
    }
    // Flush last
    if !title.is_empty() && !code_buf.is_empty() {
        records.push(ImportRecord {
            title: strip_html_tags(&title),
            description: Some(strip_html_tags(&description)),
            use_case: Some(strip_html_tags(&use_case)),
            tags: Some(strip_html_tags(&tags_str)),
            code: code_buf.trim().to_string(),
            ..Default::default()
        });
    }
    records
}

/// Very naïve HTML tag stripper — good enough for known CPKB output.
fn strip_html_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    // Also unescape basic HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

// ─── Core import driver ────────────────────────────────────────────────────

/// Insert `records` into `conn`, skipping blank entries and handling ID collisions.
///
/// `preserve_ids = true` keeps incoming IDs when they don't already exist.
/// `id_format` selects which counter sequence to use for new IDs.
pub fn import_snippets(
    conn: &mut Connection,
    records: Vec<ImportRecord>,
    preserve_ids: bool,
    id_format: Option<&str>,
) -> anyhow::Result<ImportResult> {
    let mut result = ImportResult::default();

    for record in records {
        if record.title.trim().is_empty() || record.code.trim().is_empty() {
            result.skipped += 1;
            continue;
        }

        // Determine target ID
        let target_id = if preserve_ids {
            if let Some(ref src_id) = record.id {
                if !src_id.is_empty() && !snippet_exists(conn, src_id)? {
                    src_id.clone()
                } else {
                    generate_next_id(conn, id_format)?
                }
            } else {
                generate_next_id(conn, id_format)?
            }
        } else {
            generate_next_id(conn, id_format)?
        };

        insert_snippet_with_id(
            conn,
            &target_id,
            record.title.trim(),
            record.description.as_deref().unwrap_or(""),
            record.use_case.as_deref().unwrap_or(""),
            record.tags.as_deref().unwrap_or(""),
            &record.code,
            record.language.as_deref().unwrap_or(""),
            record.created_at.as_deref(),
            record.updated_at.as_deref(),
        )?;

        result.imported += 1;
        if let Some(ref src_id) = record.id {
            result.id_map.insert(src_id.clone(), target_id);
        }
    }

    Ok(result)
}

// ─── High-level file loader ────────────────────────────────────────────────

/// Load import records from a file path, auto-detecting format.
pub fn load_from_path(path: &Path) -> anyhow::Result<Vec<ImportRecord>> {
    let path_str = path.to_string_lossy();
    let format = detect_format(&path_str);

    match format {
        ImportFormat::Db => load_db(path),
        ImportFormat::Json => {
            let bytes = std::fs::read(path)?;
            load_json(&bytes)
        }
        ImportFormat::Markdown => {
            let text = std::fs::read_to_string(path)?;
            Ok(load_markdown(&text))
        }
        ImportFormat::Html => {
            let text = std::fs::read_to_string(path)?;
            Ok(load_html(&text))
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format() {
        assert_eq!(detect_format("foo.json"), ImportFormat::Json);
        assert_eq!(detect_format("bar.md"),   ImportFormat::Markdown);
        assert_eq!(detect_format("baz.html"), ImportFormat::Html);
        assert_eq!(detect_format("qux.db"),   ImportFormat::Db);
        assert_eq!(detect_format("unknown"),  ImportFormat::Markdown);
    }

    #[test]
    fn test_load_markdown_basic() {
        let md = "## Fibonacci (CP0001)\n\
                  **Description:** Classic recursion\n\
                  **Use case:** recursion problems\n\
                  **Tags:** dp, math\n\n\
                  ```python\n\
                  def fib(n): return n if n < 2 else fib(n-1)+fib(n-2)\n\
                  ```\n";
        let records = load_markdown(md);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id.as_deref(), Some("CP0001"));
        assert_eq!(records[0].title, "Fibonacci");
        assert!(records[0].code.contains("fib"));
        assert_eq!(records[0].language.as_deref(), Some("python"));
    }

    #[test]
    fn test_load_json_basic() {
        let json = r#"[{"id":"CP0001","title":"Test","code":"print()","tags":"t1"}]"#;
        let records = load_json(json.as_bytes()).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].title, "Test");
        assert_eq!(records[0].tags.as_deref(), Some("t1"));
    }

    #[test]
    fn test_import_into_memory_db() {
        use crate::db::{get_in_memory_conn, snippets::count_snippets};
        let mut conn = get_in_memory_conn().unwrap();
        let records = vec![
            ImportRecord {
                title: "Hello".to_string(),
                code: "println!(\"hello\")".to_string(),
                ..Default::default()
            },
            ImportRecord {
                title: "".to_string(), // should be skipped
                code: "x".to_string(),
                ..Default::default()
            },
        ];
        let result = import_snippets(&mut conn, records, false, None).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(count_snippets(&conn).unwrap(), 1);
    }
}
