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
