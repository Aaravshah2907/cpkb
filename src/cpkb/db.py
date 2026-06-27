"""
CPKB Database Repository Layer
Abstracts all SQLite interactions into clean helper functions.
"""

import sqlite3
import shutil
from pathlib import Path
from datetime import datetime, timezone
import os

from .config import DEFAULT_CONFIG, load_config, max_backups, max_snippets, save_config

# XDG Base Directory specification
XDG_DATA_HOME = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share"))
APP_DIR = XDG_DATA_HOME / "cpkb"
DB_PATH = APP_DIR / "snippets.db"
KEY_PATH = APP_DIR / "encryption.key"
CURRENT_SCHEMA_VERSION = 2


# ---------------------------------------------------------------------------
# Connection & Schema
# ---------------------------------------------------------------------------

def get_conn() -> sqlite3.Connection:
    """Return a connection to the CPKB database with foreign keys enabled."""
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


def _table_exists(cursor: sqlite3.Cursor, table_name: str) -> bool:
    """Return whether *table_name* exists in the current database."""
    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name=?", (table_name,))
    return cursor.fetchone() is not None


def get_schema_version(cursor: sqlite3.Cursor) -> int:
    """Return the stored schema version, inferring legacy versions when absent."""
    if _table_exists(cursor, "schema_meta"):
        cursor.execute("SELECT value FROM schema_meta WHERE key = 'schema_version'")
        row = cursor.fetchone()
        if row and str(row[0]).isdigit():
            return int(row[0])

    if not _table_exists(cursor, "snippets"):
        return CURRENT_SCHEMA_VERSION
    if _table_exists(cursor, "reviews"):
        return CURRENT_SCHEMA_VERSION
    if _table_exists(cursor, "tags"):
        return 1
    return 0


def set_schema_version(cursor: sqlite3.Cursor, version: int) -> None:
    """Persist the database schema version."""
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
    ''')
    cursor.execute('''
        INSERT INTO schema_meta (key, value, updated_at)
        VALUES ('schema_version', ?, ?)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at
    ''', (str(version), _now()))


def _create_schema(cursor: sqlite3.Cursor) -> None:
    """Create all current-version tables if they do not exist."""
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

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS reviews (
            snippet_id TEXT PRIMARY KEY,
            ease_factor REAL NOT NULL DEFAULT 2.5,
            interval INTEGER NOT NULL DEFAULT 0,
            repetitions INTEGER NOT NULL DEFAULT 0,
            next_review TEXT NOT NULL,
            last_reviewed TEXT NOT NULL,
            FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
    ''')


def _populate_tags_from_snippets(cursor: sqlite3.Cursor) -> None:
    """Populate normalized tag rows from the legacy snippets.tags column."""
    cursor.execute("SELECT id, tags FROM snippets")
    for snip_id, tags_str in cursor.fetchall():
        if tags_str:
            for tag in [t.strip().lower() for t in tags_str.split(',') if t.strip()]:
                cursor.execute(
                    "INSERT INTO tags (snippet_id, tag) VALUES (?, ?)",
                    (snip_id, tag),
                )


def migrate_db(cursor: sqlite3.Cursor, conn: sqlite3.Connection, db_existed: bool) -> None:
    """Safely migrate old databases to the current schema version."""
    old_version = get_schema_version(cursor)
    if old_version > CURRENT_SCHEMA_VERSION:
        print(
            f"Warning: database schema version {old_version} is newer than this CPKB "
            f"supports ({CURRENT_SCHEMA_VERSION}). Please upgrade CPKB as soon as possible.",
        )
        return

    if db_existed and old_version < CURRENT_SCHEMA_VERSION:
        print(
            f"Database schema v{old_version} detected. Creating backup before "
            f"migrating to v{CURRENT_SCHEMA_VERSION}."
        )
        backup_db("pre_migration")

    had_tags = _table_exists(cursor, "tags")
    _create_schema(cursor)

    if old_version < 1 and not had_tags:
        _populate_tags_from_snippets(cursor)

    set_schema_version(cursor, CURRENT_SCHEMA_VERSION)
    conn.commit()

    if db_existed and old_version < CURRENT_SCHEMA_VERSION:
        print(f"Migration complete: schema v{old_version} -> v{CURRENT_SCHEMA_VERSION}.")


def init_db() -> sqlite3.Connection:
    """Initialize the directory structure and database schema.

    Creates all required application directories and database tables.
    Performs automatic migration for the ``tags`` table when upgrading
    from older schema versions.
    """
    APP_DIR.mkdir(parents=True, exist_ok=True)
    for sub in ("backups", "exports", "imports", "logs", "attachments"):
        (APP_DIR / sub).mkdir(exist_ok=True)
    save_config(APP_DIR, load_config(APP_DIR))

    db_exists = DB_PATH.exists()
    conn = get_conn()
    cursor = conn.cursor()

    migrate_db(cursor, conn, db_exists)
    return conn


def _now() -> str:
    """Return the current UTC time as an ISO-8601 string."""
    return datetime.now(timezone.utc).isoformat()


# ---------------------------------------------------------------------------
# ID Generation
# ---------------------------------------------------------------------------

def _id_format_config(format_name: str | None = None) -> tuple[str, int]:
    """Return the prefix and numeric width for a configured ID format."""
    config = load_config(APP_DIR)
    snippets_config = config.get("snippets", {})
    selected = format_name or snippets_config.get("default_id_format", "default")
    formats = snippets_config.get("id_formats", {})

    fallback_formats = DEFAULT_CONFIG["snippets"]["id_formats"]
    format_config = formats.get(selected) or fallback_formats["default"]
    prefix = str(format_config.get("prefix", "CP"))
    width_value = format_config.get("width", "auto")
    if width_value == "auto":
        width = len(str(max_snippets(APP_DIR)))
    else:
        try:
            width = max(1, int(width_value))
        except (TypeError, ValueError):
            width = len(str(max_snippets(APP_DIR)))

    return prefix, width


def generate_id(cursor: sqlite3.Cursor, format_name: str | None = None) -> str:
    """Generate the next snippet ID for a configured format (e.g., CP0001)."""
    prefix, width = _id_format_config(format_name)
    cursor.execute("SELECT id FROM snippets")
    existing = []
    for (snippet_id,) in cursor.fetchall():
        if not snippet_id.startswith(prefix):
            continue
        suffix = snippet_id[len(prefix):]
        if suffix.isdigit():
            existing.append(int(suffix))

    next_num = max(existing, default=0) + 1
    limit = max_snippets(APP_DIR)
    if next_num > limit:
        raise ValueError(f"Maximum snippet count reached ({limit}). Update config.json to increase it.")
    return f"{prefix}{next_num:0{width}d}"


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
                tags: str, code: str, id_format: str | None = None) -> str:
    """Insert a new snippet and return its generated ID."""
    snippet_id = generate_id(cursor, id_format)
    now = _now()
    cursor.execute('''
        INSERT INTO snippets (id, title, description, use_case, tags, code, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ''', (snippet_id, title, description, use_case, tags, code, now, now))
    update_tags(cursor, snippet_id, tags)
    conn.commit()
    return snippet_id


def insert_snippet_with_id(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
                           snippet_id: str, title: str, description: str,
                           use_case: str, tags: str, code: str,
                           created_at: str | None = None,
                           updated_at: str | None = None) -> str:
    """Insert a snippet with a caller-provided ID and return the inserted ID."""
    now = _now()
    cursor.execute('''
        INSERT INTO snippets (id, title, description, use_case, tags, code, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ''', (
        snippet_id, title, description, use_case, tags, code,
        created_at or now, updated_at or now,
    ))
    update_tags(cursor, snippet_id, tags)
    conn.commit()
    return snippet_id


def import_snippets(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
                    snippets: list[dict], preserve_ids: bool = True,
                    id_format: str | None = None) -> dict:
    """Append snippet dictionaries into the current DB without overwriting rows.

    If ``preserve_ids`` is true, incoming IDs are kept when they do not already
    exist. Missing IDs or collisions receive normal generated ``CP`` IDs.
    Returns counts and an ID mapping from source IDs to inserted IDs.
    """
    imported = 0
    skipped = 0
    id_map = {}

    for snippet in snippets:
        title = (snippet.get("title") or "").strip()
        code = (snippet.get("code") or "").strip()
        if not title or not code:
            skipped += 1
            continue

        source_id = (snippet.get("id") or "").strip()
        target_id = ""
        if preserve_ids and source_id:
            cursor.execute("SELECT 1 FROM snippets WHERE id = ?", (source_id,))
            if cursor.fetchone() is None:
                target_id = source_id

        if not target_id:
            target_id = generate_id(cursor, id_format)

        insert_snippet_with_id(
            cursor,
            conn,
            target_id,
            title,
            snippet.get("description") or "",
            snippet.get("use_case") or "",
            snippet.get("tags") or "",
            code,
            snippet.get("created_at"),
            snippet.get("updated_at"),
        )
        imported += 1
        if source_id:
            id_map[source_id] = target_id

    return {"imported": imported, "skipped": skipped, "id_map": id_map}


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


# ---------------------------------------------------------------------------
# Spaced-Repetition (SM-2) Helpers
# ---------------------------------------------------------------------------

def get_due_snippet(cursor: sqlite3.Cursor) -> tuple | None:
    """Return the most overdue snippet for review.

    Returns ``(id, title, description, code)`` for the snippet whose
    ``next_review`` date is the oldest (i.e. most overdue), or ``None``
    if no snippets are due.
    """
    now = _now()
    cursor.execute('''
        SELECT s.id, s.title, s.description, s.code
        FROM snippets s
        JOIN reviews r ON s.id = r.snippet_id
        WHERE r.next_review <= ?
        ORDER BY r.next_review ASC
        LIMIT 1
    ''', (now,))
    return cursor.fetchone()


def get_unreviewed_snippet(cursor: sqlite3.Cursor) -> tuple | None:
    """Return a random snippet that has never been reviewed, or ``None``."""
    cursor.execute('''
        SELECT s.id, s.title, s.description, s.code
        FROM snippets s
        LEFT JOIN reviews r ON s.id = r.snippet_id
        WHERE r.snippet_id IS NULL
        ORDER BY RANDOM()
        LIMIT 1
    ''')
    return cursor.fetchone()


def upsert_review(cursor: sqlite3.Cursor, conn: sqlite3.Connection,
                  snippet_id: str, quality: int) -> dict:
    """Update the SRS schedule for *snippet_id* using the SM-2 algorithm.

    *quality* is an integer from 0 to 5:
        0-2  = forgot / incorrect (resets repetitions)
        3    = correct with serious difficulty
        4    = correct with some hesitation
        5    = perfect recall

    Returns a dict with the computed ``ease_factor``, ``interval``,
    ``repetitions``, and ``next_review`` values.
    """
    from datetime import datetime, timedelta, timezone as tz

    quality = max(0, min(5, quality))

    # Fetch existing review data (if any)
    cursor.execute(
        "SELECT ease_factor, interval, repetitions FROM reviews WHERE snippet_id = ?",
        (snippet_id,),
    )
    row = cursor.fetchone()

    if row:
        ef, interval, reps = row
    else:
        ef, interval, reps = 2.5, 0, 0

    # SM-2 algorithm
    if quality >= 3:
        if reps == 0:
            interval = 1
        elif reps == 1:
            interval = 6
        else:
            interval = round(interval * ef)
        reps += 1
    else:
        reps = 0
        interval = 1

    # Update ease factor (minimum 1.3)
    ef = ef + (0.1 - (5 - quality) * (0.08 + (5 - quality) * 0.02))
    ef = max(1.3, ef)

    now = datetime.now(tz.utc)
    next_review = (now + timedelta(days=interval)).isoformat()
    now_iso = now.isoformat()

    cursor.execute('''
        INSERT INTO reviews (snippet_id, ease_factor, interval, repetitions, next_review, last_reviewed)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(snippet_id) DO UPDATE SET
            ease_factor = excluded.ease_factor,
            interval = excluded.interval,
            repetitions = excluded.repetitions,
            next_review = excluded.next_review,
            last_reviewed = excluded.last_reviewed
    ''', (snippet_id, ef, interval, reps, next_review, now_iso))
    conn.commit()

    return {
        "ease_factor": round(ef, 2),
        "interval": interval,
        "repetitions": reps,
        "next_review": next_review,
    }


def get_srs_stats(cursor: sqlite3.Cursor) -> dict:
    """Return spaced-repetition statistics."""
    now = _now()
    cursor.execute("SELECT COUNT(*) FROM snippets")
    total = cursor.fetchone()[0]
    cursor.execute("SELECT COUNT(*) FROM reviews")
    reviewed = cursor.fetchone()[0]
    cursor.execute("SELECT COUNT(*) FROM reviews WHERE next_review <= ?", (now,))
    due = cursor.fetchone()[0]
    cursor.execute("SELECT AVG(ease_factor) FROM reviews")
    avg_ef = cursor.fetchone()[0]
    return {
        "total_snippets": total,
        "reviewed": reviewed,
        "never_reviewed": total - reviewed,
        "due_now": due,
        "avg_ease_factor": round(avg_ef, 2) if avg_ef else None,
    }


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
    prune_backups()
    return str(backup_path)


def prune_backups() -> None:
    """Delete oldest backups when configured retention is exceeded."""
    limit = max_backups(APP_DIR)
    backups_dir = APP_DIR / "backups"
    if limit <= 0:
        backups = sorted(backups_dir.glob("snippets_*.db"), key=lambda p: p.stat().st_mtime)
    else:
        backups = sorted(backups_dir.glob("snippets_*.db"), key=lambda p: p.stat().st_mtime)
        backups = backups[:-limit]

    for backup in backups:
        backup.unlink(missing_ok=True)
