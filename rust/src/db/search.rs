//! Full-text and multi-keyword search queries for CPKB.

use rusqlite::{Connection, Result};
use crate::db::snippets::SnippetSummary;

/// Multi-keyword AND search returning snippet summaries.
pub fn search_snippets(conn: &Connection, query: &str) -> Result<Vec<SnippetSummary>> {
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.is_empty() {
        return crate::db::snippets::list_snippets(conn);
    }

    let mut sql = "SELECT id, title, tags FROM snippets WHERE ".to_string();
    let mut clauses = Vec::new();
    for i in 1..=words.len() {
        clauses.push(format!(
            "(id LIKE ?{i} OR title LIKE ?{i} OR description LIKE ?{i} OR use_case LIKE ?{i} OR tags LIKE ?{i} OR code LIKE ?{i})"
        ));
    }
    sql.push_str(&clauses.join(" AND "));
    sql.push_str(" ORDER BY created_at DESC");

    let patterns: Vec<String> = words.into_iter().map(|w| format!("%{}%", w)).collect();
    let params_ref: Vec<&dyn rusqlite::ToSql> = patterns.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_ref.as_slice(), |row| {
        Ok(SnippetSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            tags: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Scripting-friendly search returning pairs of (id, title) limited by `limit`.
pub fn query_snippets(conn: &Connection, query: &str, limit: u32) -> Result<Vec<(String, String)>> {
    let summaries = search_snippets(conn, query)?;
    Ok(summaries
        .into_iter()
        .take(limit as usize)
        .map(|s| (s.id, s.title))
        .collect())
}
