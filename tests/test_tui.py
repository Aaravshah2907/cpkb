import pytest
from unittest.mock import patch
import tempfile
from pathlib import Path
from cpkb.tui import SnippetApp
from cpkb import db


@pytest.fixture
def mock_db():
    """Create a temporary database for TUI tests."""
    temp_dir = tempfile.TemporaryDirectory()
    temp_path = Path(temp_dir.name) / "test.db"
    with patch.object(db, "DB_PATH", temp_path), \
         patch.object(db, "APP_DIR", Path(temp_dir.name)):
        db.init_db()
        yield
    temp_dir.cleanup()


@pytest.mark.asyncio
async def test_tui_launches(mock_db):
    """Verify the TUI launches without crashing and has expected widgets."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        assert app.query_one("#search-input") is not None
        assert app.query_one("#snippet-list") is not None
        assert app.query_one("#snippet-view") is not None


@pytest.mark.asyncio
async def test_tui_search_input(mock_db):
    """Verify typing in the search bar doesn't crash the app."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.press("/")
        await pilot.press("t", "e", "s", "t")
        await pilot.pause()
        assert True
