//! Normalized tag helpers for CPKB snippets.

use rusqlite::{params, Connection, Result};

/// Replace all tags for `snippet_id` with parsed comma-separated tags.
pub fn update_tags(conn: &Connection, snippet_id: &str, tags_str: &str) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE snippet_id = ?1", params![snippet_id])?;
    if !tags_str.trim().is_empty() {
        for tag in tags_str.split(',') {
            let clean = tag.trim().to_lowercase();
            if !clean.is_empty() {
                conn.execute(
                    "INSERT INTO tags (snippet_id, tag) VALUES (?1, ?2)",
                    params![snippet_id, clean],
                )?;
            }
        }
    }
    Ok(())
}

/// Add a single tag keyword to a snippet.
pub fn add_tag(conn: &mut Connection, snippet_id: &str, new_tag: &str) -> Result<String, Box<dyn std::error::Error>> {
    let clean = new_tag.trim().to_lowercase();
    if clean.is_empty() {
        return Err("Tag cannot be empty".into());
    }

    let current_tags: Option<String> = conn.query_row(
        "SELECT tags FROM snippets WHERE id = ?1",
        params![snippet_id],
        |row| row.get(0),
    )?;

    let mut tags: Vec<String> = current_tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if !tags.iter().any(|t| t.to_lowercase() == clean) {
        tags.push(clean.clone());
    }

    let merged = tags.join(", ");
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE snippets SET tags = ?1 WHERE id = ?2",
        params![merged, snippet_id],
    )?;
    update_tags(&tx, snippet_id, &merged)?;
    tx.commit()?;

    Ok(merged)
}

/// Remove an existing tag keyword from a snippet.
pub fn remove_tag(conn: &mut Connection, snippet_id: &str, target_tag: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let clean = target_tag.trim().to_lowercase();
    let current_tags: Option<String> = conn.query_row(
        "SELECT tags FROM snippets WHERE id = ?1",
        params![snippet_id],
        |row| row.get(0),
    )?;

    let tags_str = current_tags.unwrap_or_default();
    let mut tags: Vec<String> = tags_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let initial_len = tags.len();
    tags.retain(|t| t.to_lowercase() != clean);

    if tags.len() == initial_len {
        return Ok(None); // Tag was not present
    }

    let merged = tags.join(", ");
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE snippets SET tags = ?1 WHERE id = ?2",
        params![merged, snippet_id],
    )?;
    update_tags(&tx, snippet_id, &merged)?;
    tx.commit()?;

    Ok(Some(merged))
}
