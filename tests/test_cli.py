import pytest
from unittest.mock import patch, MagicMock
import sqlite3
import tempfile
from pathlib import Path
from cpkb import cli
from cpkb import db


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
