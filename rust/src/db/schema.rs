//! Database table schema constants and schema version metadata.

pub const CURRENT_SCHEMA_VERSION: u32 = 3;

pub const CREATE_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS snippets (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    use_case TEXT,
    tags TEXT,
    code TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'cpp',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS usages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snippet_id TEXT NOT NULL,
    file_path TEXT,
    problem_name TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snippet_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS reviews (
    snippet_id TEXT PRIMARY KEY,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    interval INTEGER NOT NULL DEFAULT 0,
    repetitions INTEGER NOT NULL DEFAULT 0,
    next_review TEXT NOT NULL,
    last_reviewed TEXT NOT NULL,
    FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS schema_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;
