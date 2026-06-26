import pytest
from unittest.mock import patch
import tempfile
from pathlib import Path
from cpkb.tui import EditTagsModal, SnippetApp
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


@pytest.mark.asyncio
async def test_tui_detail_scroll_bindings_exist(mock_db):
    """Verify detail-pane keyboard scrolling actions are registered."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        assert callable(app.action_scroll_detail_down)
        assert callable(app.action_scroll_detail_up)
        assert callable(app.action_page_detail_down)
        assert callable(app.action_page_detail_up)


@pytest.mark.asyncio
async def test_edit_tags_modal_shows_current_tags(mock_db):
    """Verify the tag editor shows current tags and add/remove controls."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        modal = EditTagsModal("graph, dp")
        app.push_screen(modal)
        await pilot.pause()
        assert modal.query_one("#current-tags-input").value == "graph, dp"
        assert modal.query_one("#add-btn") is not None
        assert modal.query_one("#remove-btn") is not None
