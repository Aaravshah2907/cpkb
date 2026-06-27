import pytest
from unittest.mock import patch, MagicMock
import sqlite3
import tempfile
import time
import sys
from pathlib import Path
from cpkb import cli
from cpkb import db
from cpkb import config as cpkb_config
from cpkb.default_snippets import default_snippets


@pytest.fixture
def temp_db():
    """Create a temporary database for each test, patching db.py paths."""
    temp_dir = tempfile.TemporaryDirectory()
    temp_path = Path(temp_dir.name) / "test.db"

    with patch.object(db, "DB_PATH", temp_path), \
         patch.object(db, "APP_DIR", Path(temp_dir.name)), \
         patch.object(cli, "DB_PATH", temp_path), \
         patch.object(cli, "APP_DIR", Path(temp_dir.name)):
        conn = db.init_db()
        yield conn
        conn.close()
    temp_dir.cleanup()


def test_cmd_add(temp_db):
    """Test that cmd_add inserts a snippet with correct metadata."""
    args = MagicMock()
    with patch("builtins.input", side_effect=["My Title", "My Desc", "My Use", "tag1, tag2"]), \
         patch("sys.stdin.readlines", return_value=["print('hello')\n"]):
        cli.cmd_add(args)

    cursor = temp_db.cursor()
    cursor.execute("SELECT id, title, description, code, tags FROM snippets")
    row = cursor.fetchone()
    assert row is not None
    assert row[1] == "My Title"
    assert row[2] == "My Desc"
    assert row[3] == "print('hello')"
    assert row[4] == "tag1, tag2"


def test_cmd_delete(temp_db):
    """Test that cmd_delete removes a snippet after confirmation."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Title", "Desc", "Use", "tag"]), \
         patch("sys.stdin.readlines", return_value=["code"]):
        cli.cmd_add(args_add)

    cursor = temp_db.cursor()
    cursor.execute("SELECT id FROM snippets")
    snippet_id = cursor.fetchone()[0]

    args_delete = MagicMock()
    args_delete.id = snippet_id
    with patch("builtins.input", side_effect=["y"]):
        cli.cmd_delete(args_delete)

    cursor.execute("SELECT id FROM snippets")
    assert cursor.fetchone() is None


def test_cmd_use(temp_db):
    """Test that cmd_use records a usage entry for a snippet."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Title", "Desc", "Use", "tag"]), \
         patch("sys.stdin.readlines", return_value=["code"]):
        cli.cmd_add(args_add)

    cursor = temp_db.cursor()
    cursor.execute("SELECT id FROM snippets")
    snippet_id = cursor.fetchone()[0]

    args_use = MagicMock()
    args_use.id = snippet_id
    args_use.file = "main.py"
    with patch("builtins.input", side_effect=["Problem 1", "Notes here"]):
        cli.cmd_use(args_use)

    cursor.execute("SELECT file_path, problem_name, notes FROM usages WHERE snippet_id = ?", (snippet_id,))
    row = cursor.fetchone()
    assert row is not None
    assert row[0] == "main.py"
    assert row[1] == "Problem 1"
    assert row[2] == "Notes here"


def test_cmd_copy(temp_db):
    """Test that cmd_copy calls the clipboard helper with the snippet code."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Title", "Desc", "Use", "tag"]), \
         patch("sys.stdin.readlines", return_value=["code to copy"]):
        cli.cmd_add(args_add)

    cursor = temp_db.cursor()
    cursor.execute("SELECT id FROM snippets")
    snippet_id = cursor.fetchone()[0]

    args_copy = MagicMock()
    args_copy.id = snippet_id
    args_copy.file = None

    with patch("cpkb.cli._copy_to_clipboard") as mock_copy:
        cli.cmd_copy(args_copy)
        mock_copy.assert_called_once_with("code to copy")


def test_cmd_list(temp_db, capsys):
    """Test that cmd_list outputs the correct snippet list."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Alpha", "Desc", "Use", "sorting"]), \
         patch("sys.stdin.readlines", return_value=["code1"]):
        cli.cmd_add(args_add)

    with patch("builtins.input", side_effect=["Beta", "Desc", "Use", "graph"]), \
         patch("sys.stdin.readlines", return_value=["code2"]):
        cli.cmd_add(args_add)

    cli.cmd_list(MagicMock())
    captured = capsys.readouterr()
    assert "Alpha" in captured.out
    assert "Beta" in captured.out


def test_cmd_search(temp_db, capsys):
    """Test that cmd_search finds snippets by keyword."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Dijkstra", "Shortest path", "Graphs", "graph, dp"]), \
         patch("sys.stdin.readlines", return_value=["dijkstra_code"]):
        cli.cmd_add(args_add)

    with patch("builtins.input", side_effect=["BFS", "Breadth first", "Traversal", "graph"]), \
         patch("sys.stdin.readlines", return_value=["bfs_code"]):
        cli.cmd_add(args_add)

    args_search = MagicMock()
    args_search.query = "graph"
    cli.cmd_search(args_search)
    captured = capsys.readouterr()
    assert "Dijkstra" in captured.out
    assert "BFS" in captured.out

    args_search.query = "dp"
    cli.cmd_search(args_search)
    captured = capsys.readouterr()
    assert "Dijkstra" in captured.out
    assert "BFS" not in captured.out


def test_cmd_stats(temp_db, capsys):
    """Test that cmd_stats reports correct counts."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Title", "Desc", "Use", "tag1, tag2"]), \
         patch("sys.stdin.readlines", return_value=["code"]):
        cli.cmd_add(args_add)

    cli.cmd_stats(MagicMock())
    captured = capsys.readouterr()
    assert "Total Snippets: 1" in captured.out
    assert "Unique Tags:    2" in captured.out


def test_cmd_config_creates_default_config(temp_db, capsys):
    """Test that cmd_config prints and persists default config."""
    cli.cmd_config(MagicMock())
    captured = capsys.readouterr()
    assert "config.json" in captured.out
    assert '"default_language": "cpp"' in captured.out
    assert (db.APP_DIR / "config.json").exists()


def test_cmd_setup_yes_creates_config_and_directories(temp_db, capsys):
    """Test non-interactive setup mirrors setup.sh defaults."""
    args = MagicMock()
    args.yes = True
    args.load_defaults = False
    args.enable_encryption = False

    cli.cmd_setup(args)

    captured = capsys.readouterr()
    assert "CPKB setup complete." in captured.out
    assert "Config written to:" in captured.out
    assert "Active Python:" in captured.out
    assert (db.APP_DIR / "config.json").exists()
    for subdir in ("backups", "exports", "imports", "logs", "attachments"):
        assert (db.APP_DIR / subdir).is_dir()


def test_cmd_setup_can_import_defaults(temp_db):
    """Test setup can load bundled defaults like setup.sh."""
    args = MagicMock()
    args.yes = True
    args.load_defaults = True
    args.enable_encryption = False

    cli.cmd_setup(args)

    cursor = temp_db.cursor()
    cursor.execute("SELECT COUNT(*) FROM snippets")
    assert cursor.fetchone()[0] == 16


def test_cmd_tui_missing_textual_reports_active_python(capsys):
    """Test TUI dependency errors point at the interpreter running cpkb."""
    with patch("cpkb.cli.importlib.util.find_spec", return_value=None):
        cli.cmd_tui(MagicMock())

    captured = capsys.readouterr()
    assert "Textual is not installed for the Python running cpkb" in captured.err
    assert f"{sys.executable} -m pip install textual" in captured.err


def test_generate_id_respects_configured_max_snippets(temp_db):
    """Test ID width and limit derive from config."""
    cpkb_config.save_config(
        db.APP_DIR,
        {
            "snippets": {"max_number": 99},
            "backups": {"max_backups": 25},
        },
    )
    cursor = temp_db.cursor()
    sid = db.add_snippet(cursor, temp_db, "Repo Test", "desc", "use", "t1", "code")
    assert sid == "CP01"


def test_backup_retention_uses_config(temp_db):
    """Test backup pruning keeps only the configured number of backups."""
    cpkb_config.save_config(
        db.APP_DIR,
        {
            "snippets": {"max_number": 9999},
            "backups": {"max_backups": 2},
        },
    )
    cursor = temp_db.cursor()
    db.add_snippet(cursor, temp_db, "Repo Test", "desc", "use", "t1", "code")

    for index in range(3):
        db.backup_db(f"manual_{index}")
        time.sleep(0.01)

    backups = sorted((db.APP_DIR / "backups").glob("snippets_*.db"))
    assert len(backups) == 2


def test_init_db_migrates_legacy_database(capsys):
    """Test legacy databases are backed up and migrated to the current schema."""
    temp_dir = tempfile.TemporaryDirectory()
    temp_path = Path(temp_dir.name) / "test.db"
    app_dir = Path(temp_dir.name)
    app_dir.mkdir(exist_ok=True)

    legacy_conn = sqlite3.connect(temp_path)
    legacy_conn.execute('''
        CREATE TABLE snippets (
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
    legacy_conn.execute('''
        INSERT INTO snippets
        VALUES ('CP0001', 'Legacy', 'desc', 'use', 'Graph, DP', 'code', 'old', 'old')
    ''')
    legacy_conn.commit()
    legacy_conn.close()

    with patch.object(db, "DB_PATH", temp_path), \
         patch.object(db, "APP_DIR", app_dir):
        conn = db.init_db()
        cursor = conn.cursor()

        assert db.get_schema_version(cursor) == db.CURRENT_SCHEMA_VERSION
        cursor.execute("SELECT tag FROM tags WHERE snippet_id = ? ORDER BY tag", ("CP0001",))
        assert cursor.fetchall() == [("dp",), ("graph",)]
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='reviews'")
        assert cursor.fetchone() is not None
        conn.close()

        backups = list((app_dir / "backups").glob("snippets_pre_migration_*.db"))
        assert len(backups) == 1

    captured = capsys.readouterr()
    assert "Migration complete" in captured.out
    temp_dir.cleanup()


def test_cmd_export_db_copies_sqlite_database(temp_db, capsys):
    """Test exporting the database as a .db file."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Exported", "Desc", "Use", "tag"]), \
         patch("sys.stdin.readlines", return_value=["code"]):
        cli.cmd_add(args_add)

    args = MagicMock()
    args.encrypted = False
    cli.cmd_export_db(args)

    captured = capsys.readouterr()
    assert "Exported database to" in captured.out

    exports = list((db.APP_DIR / "exports").glob("snippets_*.db"))
    assert len(exports) == 1

    exported_conn = sqlite3.connect(exports[0])
    row = exported_conn.execute("SELECT title FROM snippets").fetchone()
    exported_conn.close()
    assert row == ("Exported",)


def test_cmd_export_db_can_encrypt(temp_db, capsys):
    """Test encrypted DB export writes salt plus Fernet ciphertext."""
    cpkb_config.save_config(
        db.APP_DIR,
        {
            "snippets": {"max_number": 9999},
            "backups": {"max_backups": 25},
            "encryption": {"enabled": True},
        },
    )
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Secret", "Desc", "Use", "tag"]), \
         patch("sys.stdin.readlines", return_value=["code"]):
        cli.cmd_add(args_add)

    args = MagicMock()
    args.encrypted = True
    with patch("getpass.getpass", side_effect=["pw", "pw"]):
        cli.cmd_export_db(args)

    captured = capsys.readouterr()
    assert "Exported database to" in captured.out

    exports = list((db.APP_DIR / "exports").glob("snippets_*.db.enc"))
    assert len(exports) == 1
    raw = exports[0].read_bytes()
    assert len(raw) > 16
    assert raw[:16] != raw[16:32]


def test_cmd_export_db_encrypted_requires_enabled_config(temp_db, capsys):
    """Test encrypted DB export is blocked when encryption is disabled."""
    args = MagicMock()
    args.encrypted = True
    cli.cmd_export_db(args)

    captured = capsys.readouterr()
    assert "Encryption is disabled" in captured.err


def test_cmd_import_defaults_adds_cpp_cheatsheets(temp_db, capsys):
    """Test bundled C++ cheatsheets import with special IDs."""
    args = MagicMock()
    args.list_defaults = False
    args.defaults = True
    args.source = None
    args.format = None
    args.encrypted = False
    args.regenerate_ids = False

    cli.cmd_import(args)

    captured = capsys.readouterr()
    assert "Imported 16 snippet(s)" in captured.out

    cursor = temp_db.cursor()
    cursor.execute("SELECT id, title, tags, code FROM snippets WHERE id = ?", ("cp_0001",))
    row = cursor.fetchone()
    assert row is not None
    assert row[1] == "Vector Method Info"
    assert row[2] == "cheat sheet, helpful"
    assert "push_back()" in row[3]


def test_cmd_import_list_defaults_previews_without_importing(temp_db, capsys):
    """Test default cheatsheet preview does not write snippets."""
    args = MagicMock()
    args.list_defaults = True

    cli.cmd_import(args)

    captured = capsys.readouterr()
    assert "cp_0001 | Vector Method Info" in captured.out

    cursor = temp_db.cursor()
    cursor.execute("SELECT COUNT(*) FROM snippets")
    assert cursor.fetchone()[0] == 0


def test_cmd_import_json_appends_without_overwriting(temp_db, capsys):
    """Test JSON import appends and regenerates colliding IDs."""
    snippets = default_snippets(db.APP_DIR)[:1]
    result = db.import_snippets(temp_db.cursor(), temp_db, snippets)
    assert result["imported"] == 1

    json_path = db.APP_DIR / "imports" / "one_snippet.json"
    json_path.write_text(
        """[
  {
    "id": "cp_0001",
    "title": "Imported Again",
    "description": "desc",
    "use_case": "use",
    "tags": "tag",
    "code": "int main() {}"
  }
]
""",
        encoding="utf-8",
    )

    args = MagicMock()
    args.list_defaults = False
    args.defaults = False
    args.source = str(json_path)
    args.format = None
    args.encrypted = False
    args.regenerate_ids = False

    cli.cmd_import(args)

    captured = capsys.readouterr()
    assert "Imported 1 snippet(s)" in captured.out
    assert "Regenerated 1 colliding ID(s)" in captured.out

    cursor = temp_db.cursor()
    cursor.execute("SELECT id FROM snippets WHERE title = ?", ("Imported Again",))
    imported_id = cursor.fetchone()[0]
    assert imported_id.startswith("CP")
    assert imported_id != "cp_0001"


def test_cmd_import_markdown_and_html_exports(temp_db):
    """Test markdown and HTML import parsers accept CPKB export shapes."""
    md = b"""## MD Title (MD001)
**Description:** Desc
**Use case:** Use
**Tags:** tag1, tag2

```
cout << 1;
```
"""
    html = b"""<html><body><section><h2>HTML Title (HTML001)</h2><p><strong>Description:</strong> Desc</p><p><strong>Use case:</strong> Use</p><p><strong>Tags:</strong> tag</p><pre>cout &lt;&lt; 2;</pre></section><hr/></body></html>"""

    md_path = db.APP_DIR / "imports" / "snippets.md"
    html_path = db.APP_DIR / "imports" / "snippets.html"
    md_path.write_bytes(md)
    html_path.write_bytes(html)

    for source in (md_path, html_path):
        args = MagicMock()
        args.list_defaults = False
        args.defaults = False
        args.source = str(source)
        args.format = None
        args.encrypted = False
        args.regenerate_ids = False
        cli.cmd_import(args)

    cursor = temp_db.cursor()
    cursor.execute("SELECT title, code FROM snippets WHERE id = ?", ("MD001",))
    assert cursor.fetchone() == ("MD Title", "cout << 1;")
    cursor.execute("SELECT title, code FROM snippets WHERE id = ?", ("HTML001",))
    assert cursor.fetchone() == ("HTML Title", "cout << 2;")


def test_cmd_tag_add_remove(temp_db, capsys):
    """Test adding and removing tags via CLI commands."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["Title", "Desc", "Use", "existing"]), \
         patch("sys.stdin.readlines", return_value=["code"]):
        cli.cmd_add(args_add)

    cursor = temp_db.cursor()
    cursor.execute("SELECT id FROM snippets")
    snippet_id = cursor.fetchone()[0]

    # Add a tag
    args_tag = MagicMock()
    args_tag.id = snippet_id
    args_tag.tag = "newtag"
    cli.cmd_tag_add(args_tag)

    cursor.execute("SELECT tags FROM snippets WHERE id = ?", (snippet_id,))
    assert "newtag" in cursor.fetchone()[0]

    # Remove a tag
    args_tag.tag = "existing"
    cli.cmd_tag_remove(args_tag)

    cursor.execute("SELECT tags FROM snippets WHERE id = ?", (snippet_id,))
    tags_str = cursor.fetchone()[0]
    assert "existing" not in tags_str
    assert "newtag" in tags_str


def test_cmd_show(temp_db, capsys):
    """Test that cmd_show prints snippet details."""
    args_add = MagicMock()
    with patch("builtins.input", side_effect=["ShowTest", "A description", "Testing", "demo"]), \
         patch("sys.stdin.readlines", return_value=["print(42)\n"]):
        cli.cmd_add(args_add)

    cursor = temp_db.cursor()
    cursor.execute("SELECT id FROM snippets")
    snippet_id = cursor.fetchone()[0]

    args_show = MagicMock()
    args_show.id = snippet_id
    cli.cmd_show(args_show)
    captured = capsys.readouterr()
    assert "ShowTest" in captured.out
    assert "A description" in captured.out
    assert "print(42)" in captured.out


def test_db_repository_functions(temp_db):
    """Test the db.py repository layer directly."""
    cursor = temp_db.cursor()

    # add_snippet
    sid = db.add_snippet(cursor, temp_db, "Repo Test", "desc", "use", "t1, t2", "code_here")
    assert sid.startswith("CP")

    # get_snippet
    row = db.get_snippet(cursor, sid)
    assert row is not None
    assert row[1] == "Repo Test"

    # update_snippet
    db.update_snippet(cursor, temp_db, sid, "Updated", "d2", "u2", "t3", "new_code")
    row = db.get_snippet(cursor, sid)
    assert row[1] == "Updated"
    assert row[5] == "new_code"

    # search_snippets
    results = db.search_snippets(cursor, "updated")
    assert len(results) == 1

    # add_usage / get_usages
    db.add_usage(cursor, temp_db, sid, "file.py", "prob", "notes")
    usages = db.get_usages(cursor, sid)
    assert len(usages) == 1
    assert usages[0][1] == "file.py"

    # add_tag / remove_tag
    db.add_tag(cursor, temp_db, sid, "newtag")
    row = db.get_snippet_fields(cursor, sid, "tags")
    assert "newtag" in row[0]

    result = db.remove_tag(cursor, temp_db, sid, "newtag")
    assert result is not None
    row = db.get_snippet_fields(cursor, sid, "tags")
    assert "newtag" not in row[0]

    # get_stats
    stats = db.get_stats(cursor)
    assert stats["snippets"] == 1
    assert stats["usages"] == 1

    # delete_snippet
    db.delete_snippet(cursor, temp_db, sid)
    assert db.get_snippet(cursor, sid) is None
