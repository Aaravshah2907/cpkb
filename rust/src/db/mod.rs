//! SQLite database lifecycle, migrations, and module exports.

pub mod schema;
pub mod snippets;
pub mod tags;
pub mod usages;
pub mod srs;
pub mod search;

use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};

use crate::config::{default_app_dir, load_config, max_backups, save_config};
use crate::db::schema::{CREATE_SCHEMA_SQL, CURRENT_SCHEMA_VERSION};

/// Return the path to the main database file (`snippets.db`).
/// Matches the Python CPKB implementation which uses `snippets.db`.
pub fn db_path(app_dir: &Path) -> PathBuf {
    app_dir.join("snippets.db")
}

/// Open a SQLite connection to `snippets.db` with foreign keys enabled.
pub fn get_conn(app_dir: &Path) -> Result<Connection> {
    fs::create_dir_all(app_dir).ok();
    let path = db_path(app_dir);
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

/// Open an in-memory SQLite connection for testing.
pub fn get_in_memory_conn() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(CREATE_SCHEMA_SQL)?;
    set_schema_version(&conn, CURRENT_SCHEMA_VERSION)?;
    Ok(conn)
}

pub fn table_exists(conn: &Connection, table_name: &str) -> bool {
    conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
        params![table_name],
        |_| Ok(()),
    )
    .is_ok()
}

pub fn get_schema_version(conn: &Connection) -> u32 {
    if table_exists(conn, "schema_meta") {
        if let Ok(Some(val)) = conn
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
        {
            if let Ok(v) = val.parse::<u32>() {
                return v;
            }
        }
    }

    if !table_exists(conn, "snippets") {
        return CURRENT_SCHEMA_VERSION;
    }
    if table_exists(conn, "reviews") {
        return CURRENT_SCHEMA_VERSION;
    }
    if table_exists(conn, "tags") {
        return 1;
    }
    0
}

pub fn set_schema_version(conn: &Connection, version: u32) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "INSERT INTO schema_meta (key, value, updated_at)
         VALUES ('schema_version', ?1, ?2)
         ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at",
        params![version.to_string(), now],
    )?;
    Ok(())
}

fn populate_tags_from_snippets(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, tags FROM snippets")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)))?;
    for item in rows {
        let (snip_id, tags_opt) = item?;
        if let Some(tags_str) = tags_opt {
            for tag in tags_str.split(',') {
                let clean = tag.trim().to_lowercase();
                if !clean.is_empty() {
                    conn.execute(
                        "INSERT INTO tags (snippet_id, tag) VALUES (?1, ?2)",
                        params![snip_id, clean],
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Create a timestamped backup of the database in `backups/`.
pub fn backup_db(app_dir: &Path, prefix: &str) -> std::io::Result<PathBuf> {
    let backup_dir = app_dir.join("backups");
    fs::create_dir_all(&backup_dir)?;
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_file = backup_dir.join(format!("{}_{}.db.bak", prefix, timestamp));
    let source_file = db_path(app_dir);
    if source_file.exists() {
        fs::copy(&source_file, &backup_file)?;
    }
    prune_backups(app_dir).ok();
    Ok(backup_file)
}

/// Keep backups within configured `max_backups` limit.
pub fn prune_backups(app_dir: &Path) -> std::io::Result<()> {
    let backup_dir = app_dir.join("backups");
    if !backup_dir.exists() {
        return Ok(());
    }
    let limit = max_backups(app_dir) as usize;
    let mut backups: Vec<PathBuf> = fs::read_dir(&backup_dir)?
        .filter_map(|e| e.ok().map(|entry| entry.path()))
        .filter(|p| p.extension().map_or(false, |ext| ext == "bak"))
        .collect();

    backups.sort_by_key(|p| p.metadata().and_then(|m| m.modified()).ok());

    if backups.len() > limit {
        let to_remove = backups.len() - limit;
        for path in backups.iter().take(to_remove) {
            fs::remove_file(path).ok();
        }
    }
    Ok(())
}

/// Safely migrate old database schema versions to `CURRENT_SCHEMA_VERSION` (v3).
pub fn migrate_db(conn: &mut Connection, app_dir: &Path, db_existed: bool) -> Result<(), Box<dyn std::error::Error>> {
    let old_version = get_schema_version(conn);
    if old_version > CURRENT_SCHEMA_VERSION {
        eprintln!(
            "Warning: database schema version {} is newer than this CPKB supports ({}).",
            old_version, CURRENT_SCHEMA_VERSION
        );
        return Ok(());
    }

    if db_existed && old_version < CURRENT_SCHEMA_VERSION {
        backup_db(app_dir, "pre_migration").ok();
    }

    let had_tags = table_exists(conn, "tags");
    conn.execute_batch(CREATE_SCHEMA_SQL)?;

    if old_version < 1 && !had_tags {
        populate_tags_from_snippets(conn)?;
    }

    if old_version < 3 && table_exists(conn, "snippets") {
        let mut pragma_stmt = conn.prepare("PRAGMA table_info(snippets)")?;
        let col_names: Vec<String> = pragma_stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();

        if !col_names.iter().any(|c| c == "language") {
            conn.execute("ALTER TABLE snippets ADD COLUMN language TEXT NOT NULL DEFAULT 'cpp'", [])?;
        } else {
            conn.execute("UPDATE snippets SET language = 'cpp' WHERE language IS NULL OR language = ''", [])?;
        }

        conn.execute(
            "UPDATE snippets SET language = 'tex'
             WHERE (id LIKE 'LATEX-%' OR id LIKE 'latex%' OR id LIKE 'LATEX%')
             AND (language = 'cpp' OR language IS NULL OR language = '')",
            [],
        )?;
    }

    set_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
    Ok(())
}

/// Initialize app directory structure, configuration, and database connection.
pub fn init_db(app_dir: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    fs::create_dir_all(app_dir)?;
    for sub in &["backups", "exports", "imports", "logs", "attachments"] {
        fs::create_dir_all(app_dir.join(sub))?;
    }
    save_config(app_dir, &load_config(app_dir)).ok();

    let db_existed = db_path(app_dir).exists();
    let mut conn = get_conn(app_dir)?;
    migrate_db(&mut conn, app_dir, db_existed)?;
    Ok(conn)
}

/// Default global init helper.
pub fn init_default_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let dir = default_app_dir();
    init_db(&dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_db_and_crud() {
        let dir = tempdir().unwrap();
        let mut conn = init_db(dir.path()).unwrap();

        assert_eq!(get_schema_version(&conn), 3);

        // Add a snippet
        let id = snippets::add_snippet(
            &mut conn,
            dir.path(),
            "Binary Search",
            "Search in sorted array",
            "fast search",
            "algo, search",
            "def bs(): pass",
            Some("python"),
            None,
        )
        .unwrap();

        assert_eq!(id, "CP0001");

        // Fetch snippet
        let snip = snippets::get_snippet(&conn, &id).unwrap().unwrap();
        assert_eq!(snip.title, "Binary Search");
        assert_eq!(snip.language, "python");

        // Search snippet
        let results = search::search_snippets(&conn, "binary search").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "CP0001");

        // Add LaTeX snippet
        let latex_id = snippets::add_snippet(
            &mut conn,
            dir.path(),
            "Quadratic Formula",
            "Solve ax^2 + bx + c",
            "math",
            "math, latex",
            r"\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}",
            None,
            Some("latex"),
        )
        .unwrap();

        assert_eq!(latex_id, "LATEX-000001");
        let latex_snip = snippets::get_snippet(&conn, &latex_id).unwrap().unwrap();
        assert_eq!(latex_snip.language, "tex");
    }

    #[test]
    fn test_srs_sm2_workflow() {
        let dir = tempdir().unwrap();
        let mut conn = init_db(dir.path()).unwrap();

        let id = snippets::add_snippet(
            &mut conn,
            dir.path(),
            "Graph DFS",
            "Depth First Search",
            "traversal",
            "graph",
            "void dfs() {}",
            Some("cpp"),
            None,
        )
        .unwrap();

        // First review with perfect recall (5)
        let rev1 = srs::upsert_review(&mut conn, &id, 5).unwrap();
        assert_eq!(rev1.repetitions, 1);
        assert_eq!(rev1.interval, 1);
        assert!(rev1.ease_factor >= 2.5);

        // Second review with perfect recall (5)
        let rev2 = srs::upsert_review(&mut conn, &id, 5).unwrap();
        assert_eq!(rev2.repetitions, 2);
        assert_eq!(rev2.interval, 6);

        let stats = srs::get_srs_stats(&conn).unwrap();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.reviewed, 1);
        assert_eq!(stats.unreviewed, 0);
    }
}
