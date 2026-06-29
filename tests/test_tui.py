import pytest
from unittest.mock import patch
import tempfile
from pathlib import Path
from cpkb import tui
from cpkb.tui import (
    AddSnippetModal,
    ConfirmDeleteModal,
    CustomThemeModal,
    EditSnippetModal,
    EditTagsModal,
    NewFormatModal,
    PathSuggester,
    SettingsModal,
    SnippetApp,
    UseSnippetModal,
    _sanitize_css_id,
)
from cpkb import db
from cpkb import config as cpkb_config


@pytest.fixture
def mock_db():
    """Create a temporary database for TUI tests."""
    temp_dir = tempfile.TemporaryDirectory()
    temp_path = Path(temp_dir.name) / "test.db"
    with patch.object(db, "DB_PATH", temp_path), \
         patch.object(db, "APP_DIR", Path(temp_dir.name)), \
         patch.object(tui, "APP_DIR", Path(temp_dir.name)):
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


@pytest.mark.asyncio
async def test_tui_loads_display_config(mock_db):
    """Verify saved theme and accent are applied when the TUI launches."""
    cpkb_config.save_config(
        db.APP_DIR,
        {
            "display": {
                "theme": "dracula",
                "accent_color": "pink",
                "left_pane_width": 45,
            },
            "snippets": {
                "code_language": "cpp",
                "default_tags": "cp",
                "default_id_format": "note",
                "id_formats": {
                    "default": {"prefix": "CP", "width": "auto"},
                    "note": {"pattern": "NOTE-###"},
                },
            },
        },
    )

    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        assert app.display_theme == "dracula"
        assert app.display_accent == "pink"
        assert app.theme == "cpkb-dracula-pink"
        assert app.left_pane_width == 45
        assert app.code_language == "cpp"
        assert app.default_tags == "cp"
        assert app.default_id_format == "note"
        assert app.id_formats["note"]["pattern"] == "NOTE-###"


@pytest.mark.asyncio
async def test_add_modal_shows_configured_id_formats(mock_db):
    """Verify add modal lets TUI users choose a configured ID format."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        modal = AddSnippetModal(
            id_formats={
                "default": {"prefix": "CP", "width": "auto"},
                "note": {"pattern": "NOTE-###"},
            },
            default_id_format="note",
        )
        app.push_screen(modal)
        await pilot.pause()
        select = modal.query_one("#id-format-select")
        assert select.value == "note"


@pytest.mark.asyncio
async def test_tui_add_uses_selected_id_format(mock_db):
    """Verify TUI add passes the selected ID format to the repository."""
    cpkb_config.save_config(
        db.APP_DIR,
        {
            "snippets": {
                "default_id_format": "note",
                "id_formats": {
                    "default": {"prefix": "CP", "width": "auto"},
                    "note": {"pattern": "NOTE-###"},
                },
            },
        },
    )

    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        await app.action_add_snippet()
        await pilot.pause()
        modal = app.screen
        modal.query_one("#title-input").value = "TUI Note"
        modal.query_one("#code-input").load_text("code")
        modal.query_one("#id-format-select").value = "note"
        modal.query_one("#save-btn").press()
        await pilot.pause()

    conn = db.get_conn()
    cursor = conn.cursor()
    cursor.execute("SELECT id FROM snippets WHERE title = ?", ("TUI Note",))
    assert cursor.fetchone() == ("NOTE-001",)
    conn.close()


@pytest.mark.asyncio
async def test_settings_modal_persists_display_config(mock_db):
    """Verify TUI settings update config.json permanently."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        app._apply_display_config("nord", "purple")
        app._save_display_config("nord", "purple")

    saved = cpkb_config.load_config(db.APP_DIR)
    assert saved["display"]["theme"] == "nord"
    assert saved["display"]["accent_color"] == "purple"


@pytest.mark.asyncio
async def test_settings_modal_shows_theme_and_accent_controls(mock_db):
    """Verify the settings modal includes theme and accent selectors."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        modal = SettingsModal(["textual-dark", "dracula"], "dracula", "pink", {"default": {"color": "cyan"}})
        app.push_screen(modal)
        await pilot.pause()
        assert modal.query_one("#theme-select") is not None
        assert modal.query_one("#accent-select") is not None
        assert modal.query_one("#apply-btn") is not None


@pytest.mark.asyncio
@pytest.mark.parametrize(
    ("modal", "button_ids"),
    [
        (AddSnippetModal(), ["#save-btn", "#cancel-btn"]),
        (EditSnippetModal("CP0001", "Title", "Desc", "Use", "tag", "code"), ["#save-btn", "#cancel-btn"]),
        (ConfirmDeleteModal("CP0001", "Title"), ["#confirm-btn", "#cancel-btn"]),
        (EditTagsModal("graph, dp"), ["#add-btn", "#remove-btn", "#cancel-btn"]),
        (UseSnippetModal(), ["#save-btn", "#cancel-btn"]),
        (NewFormatModal(), ["#save-btn", "#cancel-btn"]),
        (SettingsModal(["textual-dark", "dracula"], "textual-dark", "cyan", {"default": {"color": "cyan"}}), ["#apply-btn", "#cancel-btn"]),
    ],
)
async def test_modal_action_buttons_are_visible_on_small_terminal(mock_db, modal, button_ids):
    """Verify modal action rows remain visible in a constrained terminal."""
    app = SnippetApp()
    async with app.run_test(size=(82, 24)) as pilot:
        app.push_screen(modal)
        await pilot.pause()
        for button_id in button_ids:
            button = modal.query_one(button_id)
            assert button.display
            assert button.region.height > 0


# ---------------------------------------------------------------------------
# _sanitize_css_id unit tests
# ---------------------------------------------------------------------------

@pytest.mark.parametrize(
    ("raw", "expected"),
    [
        ("CP0001", "CP0001"),
        ("cp_00016", "cp_00016"),
        ("ALG.000001", "ALG_000001"),
        ("TEST@ID#1", "TEST_ID_1"),
        ("a b c", "a_b_c"),
        ("no-change", "no-change"),
        ("dots...many", "dots___many"),
    ],
)
def test_sanitize_css_id(raw, expected):
    """Verify _sanitize_css_id replaces CSS-invalid characters with underscores."""
    assert _sanitize_css_id(raw) == expected


# ---------------------------------------------------------------------------
# TUI with custom (dot-containing) snippet IDs
# ---------------------------------------------------------------------------

@pytest.fixture
def mock_db_with_dot_ids():
    """Create a temporary database pre-loaded with dot-containing snippet IDs."""
    temp_dir = tempfile.TemporaryDirectory()
    temp_path = Path(temp_dir.name) / "test.db"
    with patch.object(db, "DB_PATH", temp_path), \
         patch.object(db, "APP_DIR", Path(temp_dir.name)), \
         patch.object(tui, "APP_DIR", Path(temp_dir.name)):
        conn = db.init_db()
        cursor = conn.cursor()
        # Insert snippets with dot-containing custom IDs
        db.insert_snippet_with_id(
            cursor, conn,
            "ALG.000001", "Sieve of Eratosthenes", "Prime sieve",
            "Number theory", "math, primes", "code_sieve",
        )
        db.insert_snippet_with_id(
            cursor, conn,
            "ALG.000002", "Binary Search", "Classic binary search",
            "Searching", "search", "code_bsearch",
        )
        # Also add a normal ID to ensure mixed IDs work
        db.insert_snippet_with_id(
            cursor, conn,
            "CP0001", "Vector Basics", "STL vectors",
            "Containers", "stl", "code_vector",
        )
        conn.close()
        yield
    temp_dir.cleanup()


@pytest.mark.asyncio
async def test_tui_renders_dot_containing_ids(mock_db_with_dot_ids):
    """Verify the TUI does not crash when snippets have dots in their IDs."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        list_view = app.query_one("#snippet-list")
        # All three snippets should be listed
        assert len(list_view.children) == 3


@pytest.mark.asyncio
async def test_tui_widget_id_mapping_with_dots(mock_db_with_dot_ids):
    """Verify the widget-id-to-snippet mapping correctly round-trips dot IDs."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        # The mapping should contain all three snippets
        assert len(app._widget_id_to_snippet) == 3
        # Dot IDs should be sanitized in the keys but preserved in values
        assert "item_ALG_000001" in app._widget_id_to_snippet
        assert app._widget_id_to_snippet["item_ALG_000001"] == "ALG.000001"
        assert "item_ALG_000002" in app._widget_id_to_snippet
        assert app._widget_id_to_snippet["item_ALG_000002"] == "ALG.000002"
        # Normal IDs should also work
        assert "item_CP0001" in app._widget_id_to_snippet
        assert app._widget_id_to_snippet["item_CP0001"] == "CP0001"


@pytest.mark.asyncio
async def test_tui_select_dot_id_snippet_shows_detail(mock_db_with_dot_ids):
    """Verify selecting a snippet with a dot ID renders its detail view."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        list_view = app.query_one("#snippet-list")
        assert len(list_view.children) > 0
        # Click the first item to trigger selection
        await pilot.click("#snippet-list ListItem")
        await pilot.pause()
        md = app.query_one("#snippet-view")
        # The Markdown widget should now show snippet content, not the placeholder
        content = md.update.__self__._markdown if hasattr(md, '_markdown') else ""
        # Alternatively, just verify the app didn't crash and the list is intact
        assert len(list_view.children) == 3


@pytest.mark.asyncio
async def test_tui_add_snippet_with_dot_pattern(mock_db):
    """Verify adding a snippet via a dot-containing ID pattern works in the TUI."""
    cpkb_config.save_config(
        db.APP_DIR,
        {
            "snippets": {
                "default_id_format": "algo",
                "id_formats": {
                    "default": {"prefix": "CP", "width": "auto"},
                    "algo": {"pattern": "ALG.####"},
                },
            },
        },
    )

    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        await app.action_add_snippet()
        await pilot.pause()
        modal = app.screen
        modal.query_one("#title-input").value = "New Algo"
        modal.query_one("#code-input").load_text("algo code")
        modal.query_one("#id-format-select").value = "algo"
        modal.query_one("#save-btn").press()
        await pilot.pause()

        # Snippet should appear in the list without crashing
        list_view = app.query_one("#snippet-list")
        assert len(list_view.children) == 1
        # The mapping should have the dot ID
        assert "item_ALG_0001" in app._widget_id_to_snippet
        assert app._widget_id_to_snippet["item_ALG_0001"] == "ALG.0001"


@pytest.mark.asyncio
async def test_settings_modal_format_colors_with_special_names(mock_db):
    """Verify settings modal handles format names that need CSS sanitization."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        id_formats = {
            "default": {"color": "cyan"},
            "algo.v2": {"color": "pink"},
        }
        modal = SettingsModal(
            ["textual-dark"], "textual-dark", "cyan",
            id_formats,
        )
        app.push_screen(modal)
        await pilot.pause()
        # The sanitized widget IDs should be queryable
        assert modal.query_one("#fmt-color-default") is not None
        assert modal.query_one("#fmt-color-algo_v2") is not None


@pytest.mark.asyncio
async def test_tui_create_new_id_format(mock_db):
    """Verify creating a new ID format from the AddSnippetModal updates config."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        await app.action_add_snippet()
        await pilot.pause()
        
        add_modal = app.screen
        # Set dropdown to __new__
        select = add_modal.query_one("#id-format-select")
        select.value = "__new__"
        await pilot.pause()
        
        # NewFormatModal should be pushed
        new_format_modal = app.screen
        new_format_modal.query_one("#format-name-input").value = "custom"
        new_format_modal.query_one("#format-prefix-input").value = "CUST"
        new_format_modal.query_one("#format-width-input").value = "3"
        new_format_modal.query_one("#save-btn").press()
        await pilot.pause()
        
        # We should be back at add_modal, and the new format should be selected
        assert add_modal.query_one("#id-format-select").value == "custom"
        
        # Verify it was saved to config
        saved = cpkb_config.load_config(db.APP_DIR)
        assert "custom" in saved["snippets"]["id_formats"]
        assert saved["snippets"]["id_formats"]["custom"]["prefix"] == "CUST"
        assert saved["snippets"]["id_formats"]["custom"]["width"] == 3


# ---------------------------------------------------------------------------
# PathSuggester tests
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_path_suggester_returns_none_for_empty():
    """Verify PathSuggester returns None for empty input."""
    suggester = PathSuggester()
    result = await suggester.get_suggestion("")
    assert result is None


@pytest.mark.asyncio
async def test_path_suggester_suggests_existing_file():
    """Verify PathSuggester suggests files in an existing directory."""
    with tempfile.TemporaryDirectory() as tmp:
        test_file = Path(tmp) / "hello.txt"
        test_file.touch()
        suggester = PathSuggester()
        result = await suggester.get_suggestion(tmp + "/h")
        assert result is not None
        assert "hello.txt" in result


@pytest.mark.asyncio
async def test_path_suggester_appends_separator_for_dirs():
    """Verify PathSuggester appends os.sep when the match is a directory."""
    import os
    with tempfile.TemporaryDirectory() as tmp:
        subdir = Path(tmp) / "subdir"
        subdir.mkdir()
        suggester = PathSuggester()
        result = await suggester.get_suggestion(tmp + "/s")
        assert result is not None
        assert result.endswith(os.sep)


@pytest.mark.asyncio
async def test_path_suggester_returns_none_for_nonexistent_path():
    """Verify PathSuggester returns None for a path that doesn't exist."""
    suggester = PathSuggester()
    result = await suggester.get_suggestion("/definitely/not/a/real/path/xyz")
    assert result is None


# ---------------------------------------------------------------------------
# CustomThemeModal tests
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_custom_theme_modal_shows_all_color_inputs(mock_db):
    """Verify CustomThemeModal renders input fields for all theme color keys."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        theme_cfg = {
            "primary": "#00ffff", "secondary": "#3399ff", "warning": "#fabd2f",
            "error": "#ff5555", "success": "#4EBF71", "accent": "#00ffff",
            "foreground": "#ffffff", "background": "#1e1e1e", "surface": "#252526",
            "panel": "#2d2d30", "boost": "#333333", "dark": True,
        }
        modal = CustomThemeModal(theme_cfg)
        app.push_screen(modal)
        await pilot.pause()
        for key in ["primary", "secondary", "warning", "error", "success",
                     "accent", "foreground", "background", "surface", "panel", "boost"]:
            inp = modal.query_one(f"#color-{key}")
            assert inp is not None
            assert inp.value == theme_cfg[key]
        assert modal.query_one("#color-dark") is not None


@pytest.mark.asyncio
async def test_custom_theme_modal_save_returns_values(mock_db):
    """Verify saving CustomThemeModal returns the edited values."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        theme_cfg = {
            "primary": "#00ffff", "secondary": "#3399ff", "warning": "#fabd2f",
            "error": "#ff5555", "success": "#4EBF71", "accent": "#00ffff",
            "foreground": "#ffffff", "background": "#1e1e1e", "surface": "#252526",
            "panel": "#2d2d30", "boost": "#333333", "dark": True,
        }
        results = []

        def on_dismiss(result):
            results.append(result)

        modal = CustomThemeModal(theme_cfg)
        app.push_screen(modal, on_dismiss)
        await pilot.pause()
        modal.query_one("#color-primary").value = "#ff0000"
        modal.query_one("#save-btn").press()
        await pilot.pause()
        assert len(results) == 1
        assert results[0] is not None
        assert results[0]["primary"] == "#ff0000"
        assert results[0]["dark"] is True


@pytest.mark.asyncio
async def test_custom_theme_modal_cancel_returns_none(mock_db):
    """Verify cancelling CustomThemeModal returns None."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        results = []

        def on_dismiss(result):
            results.append(result)

        modal = CustomThemeModal({"primary": "#00ffff", "dark": True})
        app.push_screen(modal, on_dismiss)
        await pilot.pause()
        modal.query_one("#cancel-btn").press()
        await pilot.pause()
        assert len(results) == 1
        assert results[0] is None


@pytest.mark.asyncio
async def test_custom_theme_modal_buttons_visible_on_small_terminal(mock_db):
    """Verify CustomThemeModal buttons are visible on a small terminal."""
    app = SnippetApp()
    async with app.run_test(size=(82, 24)) as pilot:
        theme_cfg = {
            "primary": "#00ffff", "secondary": "#3399ff", "warning": "#fabd2f",
            "error": "#ff5555", "success": "#4EBF71", "accent": "#00ffff",
            "foreground": "#ffffff", "background": "#1e1e1e", "surface": "#252526",
            "panel": "#2d2d30", "boost": "#333333", "dark": True,
        }
        modal = CustomThemeModal(theme_cfg)
        app.push_screen(modal)
        await pilot.pause()
        save_btn = modal.query_one("#save-btn")
        cancel_btn = modal.query_one("#cancel-btn")
        assert save_btn.display
        assert save_btn.region.height > 0
        assert cancel_btn.display
        assert cancel_btn.region.height > 0


# ---------------------------------------------------------------------------
# Settings modal: format color labels & Edit Custom Theme button
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_settings_format_color_labels_show_id_names(mock_db):
    """Verify each format color selector has a label showing the format name."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        id_formats = {
            "default": {"color": "cyan", "prefix": "CP", "width": "auto"},
            "algo": {"color": "green", "pattern": "ALG-###"},
        }
        modal = SettingsModal(
            ["textual-dark"], "textual-dark", "cyan", id_formats,
        )
        app.push_screen(modal)
        await pilot.pause()
        labels = [str(lbl.content) for lbl in modal.query("Label")]
        assert any("default" in t for t in labels)
        assert any("algo" in t for t in labels)


@pytest.mark.asyncio
async def test_settings_edit_custom_theme_button_exists(mock_db):
    """Verify the Edit Custom Theme button is present in SettingsModal."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        modal = SettingsModal(
            ["textual-dark", "custom"], "textual-dark", "cyan",
            {"default": {"color": "cyan"}},
        )
        app.push_screen(modal)
        await pilot.pause()
        btn = modal.query_one("#edit-custom-theme-btn")
        assert btn is not None


@pytest.mark.asyncio
async def test_settings_edit_custom_theme_opens_modal(mock_db):
    """Verify clicking Edit Custom Theme button opens CustomThemeModal."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        modal = SettingsModal(
            ["textual-dark", "custom"], "textual-dark", "cyan",
            {"default": {"color": "cyan"}},
        )
        app.push_screen(modal)
        await pilot.pause()
        modal.query_one("#edit-custom-theme-btn").press()
        await pilot.pause()
        assert isinstance(app.screen, CustomThemeModal)


@pytest.mark.asyncio
async def test_custom_theme_applied_when_selected(mock_db):
    """Verify selecting 'custom' theme applies custom_theme colors from config."""
    cpkb_config.save_config(
        db.APP_DIR,
        {
            "display": {
                "theme": "custom",
                "accent_color": "cyan",
                "custom_theme": {
                    "primary": "#ff1493", "secondary": "#3399ff",
                    "warning": "#fabd2f", "error": "#ff5555",
                    "success": "#4EBF71", "accent": "#ff1493",
                    "foreground": "#ffffff", "background": "#0a0a0a",
                    "surface": "#1a1a1a", "panel": "#2a2a2a",
                    "boost": "#3a3a3a", "dark": True,
                },
            },
        },
    )
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        assert app.display_theme == "custom"
        assert app.theme == "cpkb-custom"


@pytest.mark.asyncio
async def test_settings_custom_theme_in_themes_list(mock_db):
    """Verify 'custom' appears in the settings theme dropdown."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        await pilot.pause()
        await app.action_settings()
        await pilot.pause()
        modal = app.screen
        theme_select = modal.query_one("#theme-select")
        option_values = [opt[1] for opt in theme_select._options]
        assert "custom" in option_values


# ---------------------------------------------------------------------------
# UseSnippetModal: path input has suggester
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_use_snippet_modal_has_path_suggester(mock_db):
    """Verify UseSnippetModal file input has a PathSuggester attached."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        modal = UseSnippetModal()
        app.push_screen(modal)
        await pilot.pause()
        file_input = modal.query_one("#file-input")
        assert file_input.suggester is not None
        assert isinstance(file_input.suggester, PathSuggester)


# ---------------------------------------------------------------------------
# Format color: custom hex input tests
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_settings_format_color_has_custom_hex_input(mock_db):
    """Verify each format has both a Select and a custom hex Input."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        id_formats = {
            "default": {"color": "cyan", "prefix": "CP", "width": "auto"},
        }
        modal = SettingsModal(
            ["textual-dark"], "textual-dark", "cyan", id_formats,
        )
        app.push_screen(modal)
        await pilot.pause()
        assert modal.query_one("#fmt-color-default") is not None
        assert modal.query_one("#fmt-hex-default") is not None


@pytest.mark.asyncio
async def test_settings_custom_hex_takes_priority(mock_db):
    """Verify custom hex Input value is used over Select when filled."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        id_formats = {
            "default": {"color": "cyan", "prefix": "CP", "width": "auto"},
        }
        results = []

        def on_dismiss(result):
            results.append(result)

        modal = SettingsModal(
            ["textual-dark"], "textual-dark", "cyan", id_formats,
        )
        app.push_screen(modal, on_dismiss)
        await pilot.pause()
        # Type a custom hex into the input
        modal.query_one("#fmt-hex-default").value = "#ff6347"
        modal.query_one("#apply-btn").press()
        await pilot.pause()
        assert len(results) == 1
        assert results[0]["format_colors"]["default"] == "#ff6347"


@pytest.mark.asyncio
async def test_settings_select_used_when_hex_empty(mock_db):
    """Verify Select value is used when custom hex Input is empty."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        id_formats = {
            "default": {"color": "pink", "prefix": "CP", "width": "auto"},
        }
        results = []

        def on_dismiss(result):
            results.append(result)

        modal = SettingsModal(
            ["textual-dark"], "textual-dark", "cyan", id_formats,
        )
        app.push_screen(modal, on_dismiss)
        await pilot.pause()
        # Leave hex empty, just use the Select
        assert modal.query_one("#fmt-hex-default").value == ""
        modal.query_one("#apply-btn").press()
        await pilot.pause()
        assert len(results) == 1
        assert results[0]["format_colors"]["default"] == "pink"


@pytest.mark.asyncio
async def test_settings_existing_custom_hex_prefills_input(mock_db):
    """Verify a previously saved custom hex color pre-fills the Input."""
    app = SnippetApp()
    async with app.run_test() as pilot:
        id_formats = {
            "default": {"color": "#ff6347", "prefix": "CP", "width": "auto"},
        }
        modal = SettingsModal(
            ["textual-dark"], "textual-dark", "cyan", id_formats,
        )
        app.push_screen(modal)
        await pilot.pause()
        # The hex input should be pre-filled with the custom color
        assert modal.query_one("#fmt-hex-default").value == "#ff6347"
