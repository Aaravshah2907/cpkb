"""
CPKB Database Repository Layer
Abstracts all SQLite interactions into clean helper functions.
"""

import sqlite3
import shutil
from pathlib import Path
from datetime import datetime, timezone
import os

# XDG Base Directory specification
XDG_DATA_HOME = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share"))
APP_DIR = XDG_DATA_HOME / "cpkb"
DB_PATH = APP_DIR / "snippets.db"
KEY_PATH = APP_DIR / "encryption.key"


# ---------------------------------------------------------------------------
# Connection & Schema
# ---------------------------------------------------------------------------

def get_conn() -> sqlite3.Connection:
    """Return a connection to the CPKB database with foreign keys enabled."""
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


def init_db() -> sqlite3.Connection:
    """Initialize the directory structure and database schema.

    Creates all required application directories and database tables.
    Performs automatic migration for the ``tags`` table when upgrading
    from older schema versions.
    """
    APP_DIR.mkdir(parents=True, exist_ok=True)
    for sub in ("backups", "exports", "imports", "logs", "attachments"):
        (APP_DIR / sub).mkdir(exist_ok=True)

    db_exists = DB_PATH.exists()
    conn = get_conn()
    cursor = conn.cursor()

    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='tags'")
    tags_exists = cursor.fetchone() is not None

    if db_exists and not tags_exists:
        print("Migrating database... Creating backup first.")
        backup_db("pre_migration")

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS snippets (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            use_case TEXT,
            tags TEXT,
            code TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS usages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snippet_id TEXT NOT NULL,
            file_path TEXT,
            problem_name TEXT,
            notes TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snippet_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
        )
    ''')

    if db_exists and not tags_exists:
        cursor.execute("SELECT id, tags FROM snippets")
        for snip_id, tags_str in cursor.fetchall():
            if tags_str:
                for tag in [t.strip().lower() for t in tags_str.split(',') if t.strip()]:
                    cursor.execute("INSERT INTO tags (snippet_id, tag) VALUES (?, ?)", (snip_id, tag))
        print("Migration complete: populated tags table.")

    conn.commit()
    return conn


def _now() -> str:
    """Return the current UTC time as an ISO-8601 string."""
    return datetime.now(timezone.utc).isoformat()


# ---------------------------------------------------------------------------
# ID Generation
# ---------------------------------------------------------------------------

def generate_id(cursor: sqlite3.Cursor) -> str:
    """Generate the next snippet ID (e.g., CP0001)."""
    cursor.execute("SELECT id FROM snippets ORDER BY id DESC LIMIT 1")
    row = cursor.fetchone()
    if not row:
        return "CP0001"

    last_id = row[0]
    if last_id.startswith("CP") and last_id[2:].isdigit():
        num = int(last_id[2:])
        return f"CP{num + 1:04d}"

    cursor.execute("SELECT COUNT(*) FROM snippets")
    return f"CP{cursor.fetchone()[0] + 1:04d}"


# ---------------------------------------------------------------------------
# Tag Helpers
# ---------------------------------------------------------------------------

def update_tags(cursor: sqlite3.Cursor, snippet_id: str, tags_str: str) -> None:
    """Replace all tags for *snippet_id* with those parsed from *tags_str*."""
    cursor.execute("DELETE FROM tags WHERE snippet_id = ?", (snippet_id,))
    if tags_str:
        for tag in [t.strip().lower() for t in tags_str.split(',') if t.strip()]:
            cursor.execute("INSERT INTO tags (snippet_id, tag) VALUES (?, ?)", (snippet_id, tag))


# ---------------------------------------------------------------------------
# Snippet CRUD
# ---------------------------------------------------------------------------

def add_snippet(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
                title: str, description: str, use_case: str,
                tags: str, code: str) -> str:
    """Insert a new snippet and return its generated ID."""
    snippet_id = generate_id(cursor)
    now = _now()
    cursor.execute('''
        INSERT INTO snippets (id, title, description, use_case, tags, code, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ''', (snippet_id, title, description, use_case, tags, code, now, now))
    update_tags(cursor, snippet_id, tags)
    conn.commit()
    return snippet_id


def get_snippet(cursor: sqlite3.Cursor, snippet_id: str) -> tuple | None:
    """Return the full snippet row or ``None``."""
    cursor.execute("SELECT * FROM snippets WHERE id = ?", (snippet_id,))
    return cursor.fetchone()


def get_snippet_fields(cursor: sqlite3.Cursor, snippet_id: str,
                       fields: str = "title, description, use_case, tags, code") -> tuple | None:
    """Return selected fields for a snippet or ``None``."""
    cursor.execute(f"SELECT {fields} FROM snippets WHERE id = ?", (snippet_id,))
    return cursor.fetchone()


def update_snippet(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
                   snippet_id: str, title: str, description: str,
                   use_case: str, tags: str, code: str) -> None:
    """Update an existing snippet's metadata, code, and tags."""
    now = _now()
    cursor.execute('''
        UPDATE snippets
        SET title = ?, description = ?, use_case = ?, tags = ?, code = ?, updated_at = ?
        WHERE id = ?
    ''', (title, description, use_case, tags, code, now, snippet_id))
    update_tags(cursor, snippet_id, tags)
    conn.commit()


def delete_snippet(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
                   snippet_id: str) -> None:
    """Delete a snippet by ID (cascades to usages and tags)."""
    backup_db("pre_delete")
    cursor.execute("DELETE FROM snippets WHERE id = ?", (snippet_id,))
    conn.commit()


def list_snippets(cursor: sqlite3.Cursor) -> list[tuple]:
    """Return ``(id, title, tags)`` for every snippet, newest first."""
    cursor.execute("SELECT id, title, tags FROM snippets ORDER BY created_at DESC")
    return cursor.fetchall()


def recent_snippets(cursor: sqlite3.Cursor, limit: int = 10) -> list[tuple]:
    """Return the *limit* most recently created snippets."""
    cursor.execute("SELECT id, title, tags FROM snippets ORDER BY created_at DESC LIMIT ?", (limit,))
    return cursor.fetchall()


def search_snippets(cursor: sqlite3.Cursor, query: str) -> list[tuple]:
    """Full-text search across title, description, tags, id, and code.

    All whitespace-separated words in *query* must match (AND logic).
    Returns ``(id, title, tags)`` tuples.
    """
    query_parts = query.lower().split()
    conditions = []
    params = []
    for part in query_parts:
        like_str = f"%{part}%"
        conditions.append(
            "(LOWER(title) LIKE ? OR LOWER(description) LIKE ? "
            "OR LOWER(tags) LIKE ? OR LOWER(code) LIKE ? OR LOWER(id) LIKE ?)"
        )
        params.extend([like_str, like_str, like_str, like_str, like_str])

    sql = ("SELECT id, title, tags FROM snippets WHERE "
           + " AND ".join(conditions)
           + " ORDER BY created_at DESC")
    cursor.execute(sql, params)
    return cursor.fetchall()


def search_snippets_full(cursor: sqlite3.Cursor, query: str) -> list[tuple]:
    """Like :func:`search_snippets` but returns ``(id, title)`` — used by the TUI."""
    query_parts = query.lower().split()
    conditions = []
    params = []
    for part in query_parts:
        like_str = f"%{part}%"
        conditions.append(
            "(LOWER(title) LIKE ? OR LOWER(description) LIKE ? "
            "OR LOWER(tags) LIKE ? OR LOWER(code) LIKE ? OR LOWER(id) LIKE ?)"
        )
        params.extend([like_str, like_str, like_str, like_str, like_str])

    sql = ("SELECT id, title FROM snippets WHERE "
           + " AND ".join(conditions)
           + " ORDER BY created_at DESC")
    cursor.execute(sql, params)
    return cursor.fetchall()


def get_random_snippet(cursor: sqlite3.Cursor) -> tuple | None:
    """Return a random ``(id, title, description, code)`` tuple or ``None``."""
    cursor.execute("SELECT id, title, description, code FROM snippets ORDER BY RANDOM() LIMIT 1")
    return cursor.fetchone()


def get_all_snippet_ids(cursor: sqlite3.Cursor) -> list[tuple]:
    """Return all snippet IDs."""
    cursor.execute("SELECT id FROM snippets")
    return cursor.fetchall()


# ---------------------------------------------------------------------------
# Usage CRUD
# ---------------------------------------------------------------------------

def add_usage(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
              snippet_id: str, file_path: str,
              problem_name: str = "", notes: str = "") -> None:
    """Record a usage of a snippet."""
    now = _now()
    cursor.execute('''
        INSERT INTO usages (snippet_id, file_path, problem_name, notes, created_at)
        VALUES (?, ?, ?, ?, ?)
    ''', (snippet_id, file_path, problem_name, notes, now))
    conn.commit()


def get_usages(cursor: sqlite3.Cursor, snippet_id: str) -> list[tuple]:
    """Return all usage records for a snippet, newest first."""
    cursor.execute('''
        SELECT id, file_path, problem_name, notes, created_at
        FROM usages
        WHERE snippet_id = ?
        ORDER BY created_at DESC
    ''', (snippet_id,))
    return cursor.fetchall()


def get_usage(cursor: sqlite3.Cursor, usage_id: int) -> tuple | None:
    """Return a single usage record by its ID."""
    cursor.execute("SELECT file_path, problem_name, notes FROM usages WHERE id = ?", (usage_id,))
    return cursor.fetchone()


def update_usage(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
                 usage_id: int, file_path: str,
                 problem_name: str, notes: str) -> None:
    """Update an existing usage record."""
    cursor.execute('''
        UPDATE usages
        SET file_path = ?, problem_name = ?, notes = ?
        WHERE id = ?
    ''', (file_path, problem_name, notes, usage_id))
    conn.commit()


# ---------------------------------------------------------------------------
# Tag Management
# ---------------------------------------------------------------------------

def add_tag(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
            snippet_id: str, tag: str) -> str:
    """Add a tag to a snippet. Returns the updated tags string."""
    cursor.execute("SELECT tags FROM snippets WHERE id = ?", (snippet_id,))
    row = cursor.fetchone()
    if not row:
        raise ValueError(f"Snippet {snippet_id} not found")
    existing = row[0] or ""
    tags_set = {t.strip().lower() for t in existing.split(',') if t.strip()}
    tags_set.add(tag.strip().lower())
    new_tags = ", ".join(sorted(tags_set))
    cursor.execute("UPDATE snippets SET tags = ?, updated_at = ? WHERE id = ?",
                   (new_tags, _now(), snippet_id))
    update_tags(cursor, snippet_id, new_tags)
    conn.commit()
    return new_tags


def remove_tag(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
               snippet_id: str, tag: str) -> str | None:
    """Remove a tag from a snippet. Returns the updated tags string, or ``None`` if the tag was not found."""
    cursor.execute("SELECT tags FROM snippets WHERE id = ?", (snippet_id,))
    row = cursor.fetchone()
    if not row:
        raise ValueError(f"Snippet {snippet_id} not found")
    existing = row[0] or ""
    tags_set = {t.strip().lower() for t in existing.split(',') if t.strip()}
    normalised = tag.strip().lower()
    if normalised not in tags_set:
        return None
    tags_set.remove(normalised)
    new_tags = ", ".join(sorted(tags_set))
    cursor.execute("UPDATE snippets SET tags = ?, updated_at = ? WHERE id = ?",
                   (new_tags, _now(), snippet_id))
    update_tags(cursor, snippet_id, new_tags)
    conn.commit()
    return new_tags


# ---------------------------------------------------------------------------
# Statistics
# ---------------------------------------------------------------------------

def get_stats(cursor: sqlite3.Cursor) -> dict:
    """Return basic knowledge-base statistics."""
    cursor.execute("SELECT COUNT(*) FROM snippets")
    snippet_count = cursor.fetchone()[0]
    cursor.execute("SELECT COUNT(*) FROM usages")
    usage_count = cursor.fetchone()[0]
    cursor.execute("SELECT COUNT(DISTINCT tag) FROM tags")
    tag_count = cursor.fetchone()[0]
    return {"snippets": snippet_count, "usages": usage_count, "tags": tag_count}


# ---------------------------------------------------------------------------
# Backup
# ---------------------------------------------------------------------------

def backup_db(prefix: str = "manual") -> str:
    """Create a timestamped copy of the database. Returns the backup path (empty string if no DB)."""
    if not DB_PATH.exists():
        return ""
    backups_dir = APP_DIR / "backups"
    backups_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y-%m-%d_%H%M%S")
    backup_path = backups_dir / f"snippets_{prefix}_{timestamp}.db"
    shutil.copy2(DB_PATH, backup_path)
    return str(backup_path)
