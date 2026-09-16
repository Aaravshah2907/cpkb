//! JSON export: serialises all snippets to a timestamped `.json` file.

use std::path::Path;
use chrono::Utc;
use rusqlite::Connection;
use serde_json::{json, Value};

use crate::db::snippets::list_all_snippets;

/// Export every snippet to `<app_dir>/exports/snippets_<timestamp>.json`.
///
/// Returns the number of snippets written and the output path.
pub fn export_json(conn: &Connection, app_dir: &Path) -> anyhow::Result<(usize, std::path::PathBuf)> {
    let export_dir = app_dir.join("exports");
    std::fs::create_dir_all(&export_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let out_path = export_dir.join(format!("snippets_{timestamp}.json"));

    let snippets = list_all_snippets(conn)?;
    let count = snippets.len();

    let data: Vec<Value> = snippets
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "title": s.title,
                "description": s.description,
                "use_case": s.use_case,
                "tags": s.tags,
                "code": s.code,
                "language": s.language,
                "created_at": s.created_at,
                "updated_at": s.updated_at,
            })
        })
        .collect();

    let output = serde_json::to_string_pretty(&data)?;
    std::fs::write(&out_path, output)?;
    Ok((count, out_path))
}
