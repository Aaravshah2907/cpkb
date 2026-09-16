//! Problem and file usage tracking for CPKB snippets.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub id: i64,
    pub snippet_id: String,
    pub file_path: String,
    pub problem_name: String,
    pub notes: String,
    pub created_at: String,
}

/// Record a usage instance of a snippet.
pub fn add_usage(
    conn: &Connection,
    snippet_id: &str,
    file_path: &str,
    problem_name: &str,
    notes: &str,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO usages (snippet_id, file_path, problem_name, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![snippet_id, file_path, problem_name, notes, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Retrieve all usage history for a snippet.
pub fn get_usages(conn: &Connection, snippet_id: &str) -> Result<Vec<Usage>> {
    let mut stmt = conn.prepare(
        "SELECT id, snippet_id, file_path, problem_name, notes, created_at
         FROM usages WHERE snippet_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![snippet_id], |row| {
        Ok(Usage {
            id: row.get(0)?,
            snippet_id: row.get(1)?,
            file_path: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            problem_name: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            notes: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            created_at: row.get(5)?,
        })
    })?;

    let mut usages = Vec::new();
    for r in rows {
        usages.push(r?);
    }
    Ok(usages)
}

/// Retrieve a single usage record by ID.
pub fn get_usage(conn: &Connection, usage_id: i64) -> Result<Option<Usage>> {
    conn.query_row(
        "SELECT id, snippet_id, file_path, problem_name, notes, created_at
         FROM usages WHERE id = ?1",
        params![usage_id],
        |row| {
            Ok(Usage {
                id: row.get(0)?,
                snippet_id: row.get(1)?,
                file_path: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                problem_name: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                notes: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                created_at: row.get(5)?,
            })
        },
    )
    .optional()
}

/// Update an existing usage record.
pub fn update_usage(
    conn: &Connection,
    usage_id: i64,
    file_path: &str,
    problem_name: &str,
    notes: &str,
) -> Result<bool> {
    let rows = conn.execute(
        "UPDATE usages SET file_path = ?1, problem_name = ?2, notes = ?3 WHERE id = ?4",
        params![file_path, problem_name, notes, usage_id],
    )?;
    Ok(rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{get_in_memory_conn, snippets::insert_snippet_with_id};

    fn seed_snippet(conn: &mut rusqlite::Connection) -> String {
        let id = "S1".to_string();
        insert_snippet_with_id(conn, &id, "Test", "", "", "", "code", "cpp", None, None).unwrap();
        id
    }

    #[test]
    fn test_add_usage_returns_positive_id() {
        let mut conn = get_in_memory_conn().unwrap();
        let snip_id = seed_snippet(&mut conn);
        let usage_id = add_usage(&conn, &snip_id, "main.cpp", "Prob A", "notes").unwrap();
        assert!(usage_id > 0);
    }

    #[test]
    fn test_get_usages_returns_all_for_snippet() {
        let mut conn = get_in_memory_conn().unwrap();
        let snip_id = seed_snippet(&mut conn);

        add_usage(&conn, &snip_id, "a.cpp", "Prob A", "").unwrap();
        add_usage(&conn, &snip_id, "b.cpp", "Prob B", "").unwrap();
        add_usage(&conn, &snip_id, "c.cpp", "Prob C", "").unwrap();

        let usages = get_usages(&conn, &snip_id).unwrap();
        assert_eq!(usages.len(), 3);
    }

    #[test]
    fn test_get_usage_by_id() {
        let mut conn = get_in_memory_conn().unwrap();
        let snip_id = seed_snippet(&mut conn);

        let uid = add_usage(&conn, &snip_id, "x.cpp", "MyProblem", "important").unwrap();
        let usage = get_usage(&conn, uid).unwrap().unwrap();

        assert_eq!(usage.file_path, "x.cpp");
        assert_eq!(usage.problem_name, "MyProblem");
        assert_eq!(usage.notes, "important");
        assert_eq!(usage.snippet_id, snip_id);
    }

    #[test]
    fn test_get_usage_nonexistent_returns_none() {
        let conn = get_in_memory_conn().unwrap();
        assert!(get_usage(&conn, 99999).unwrap().is_none());
    }

    #[test]
    fn test_update_usage_changes_fields() {
        let mut conn = get_in_memory_conn().unwrap();
        let snip_id = seed_snippet(&mut conn);

        let uid = add_usage(&conn, &snip_id, "old.cpp", "Old Prob", "old note").unwrap();
        let updated = update_usage(&conn, uid, "new.cpp", "New Prob", "new note").unwrap();
        assert!(updated);

        let usage = get_usage(&conn, uid).unwrap().unwrap();
        assert_eq!(usage.file_path, "new.cpp");
        assert_eq!(usage.problem_name, "New Prob");
        assert_eq!(usage.notes, "new note");
    }

    #[test]
    fn test_update_nonexistent_usage_returns_false() {
        let conn = get_in_memory_conn().unwrap();
        let updated = update_usage(&conn, 99999, "x.cpp", "X", "").unwrap();
        assert!(!updated);
    }

    #[test]
    fn test_get_usages_empty_for_new_snippet() {
        let mut conn = get_in_memory_conn().unwrap();
        let snip_id = seed_snippet(&mut conn);
        let usages = get_usages(&conn, &snip_id).unwrap();
        assert!(usages.is_empty());
    }
}

