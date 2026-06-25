import pytest
from unittest.mock import patch, MagicMock
import sqlite3
import tempfile
from pathlib import Path
from cpkb import cli

@pytest.fixture
def temp_db():
    temp_dir = tempfile.TemporaryDirectory()
    temp_path = Path(temp_dir.name) / "test.db"
    
    with patch("cpkb.cli.DB_PATH", temp_path), \
         patch("cpkb.cli.APP_DIR", Path(temp_dir.name)):
        conn = cli.init_db()
        yield conn
        conn.close()
    temp_dir.cleanup()

def test_cmd_add(temp_db):
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
