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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::db::{get_in_memory_conn, snippets::insert_snippet_with_id};

    fn seed(conn: &mut rusqlite::Connection, id: &str, title: &str, lang: &str, code: &str) {
        insert_snippet_with_id(conn, id, title, "desc", "use", "tag", code, lang, None, None).unwrap();
    }

    #[test]
    fn test_export_json_empty_db_produces_empty_array() {
        let dir = tempdir().unwrap();
        let conn = get_in_memory_conn().unwrap();
        let (count, path) = export_json(&conn, dir.path()).unwrap();
        assert_eq!(count, 0);
        // File is created but contains an empty JSON array
        let content = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_export_json_round_trip_titles_and_ids() {
        let dir = tempdir().unwrap();
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "CP0001", "Alpha", "cpp", "a()");
        seed(&mut conn, "CP0002", "Beta",  "py",  "b()");

        let (count, path) = export_json(&conn, dir.path()).unwrap();
        assert_eq!(count, 2);
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        let arr: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(arr.len(), 2);

        // Should be ordered by created_at ASC (CP0001 first)
        assert_eq!(arr[0]["id"], "CP0001");
        assert_eq!(arr[0]["title"], "Alpha");
        assert_eq!(arr[1]["id"], "CP0002");
        assert_eq!(arr[1]["title"], "Beta");
    }

    #[test]
    fn test_export_json_includes_language_field() {
        let dir = tempdir().unwrap();
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "CP0001", "Snippet", "cpp", "int main(){}");

        let (_n, path) = export_json(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let arr: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(arr[0]["language"], "cpp");
    }

    #[test]
    fn test_export_json_output_is_pretty_printed() {
        let dir = tempdir().unwrap();
        let mut conn = get_in_memory_conn().unwrap();
        seed(&mut conn, "CP0001", "X", "rust", "fn x(){}");

        let (_n, path) = export_json(&conn, dir.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        // Pretty-printed JSON has newlines and indentation
        assert!(content.contains('\n'));
        assert!(content.contains("  "));
    }
}
