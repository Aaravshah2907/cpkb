import pytest
from textual.app import App
from cpkb.tui import SnippetApp
from unittest.mock import patch
import tempfile
from pathlib import Path

@pytest.fixture
def mock_db():
    temp_dir = tempfile.TemporaryDirectory()
    temp_path = Path(temp_dir.name) / "test.db"
    with patch("cpkb.cli.DB_PATH", temp_path), \
         patch("cpkb.cli.APP_DIR", Path(temp_dir.name)):
        yield
    temp_dir.cleanup()

@pytest.mark.asyncio
async def test_tui_search(mock_db):
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        assert app.query_one("#search-input") is not None
        assert app.query_one("#snippet-list") is not None
        
        await pilot.press("/")
        await pilot.press("t", "e", "s", "t")
        await pilot.pause()
        
        # Ensure it doesn't crash on simple inputs
        assert True
