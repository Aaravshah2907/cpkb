import re as _re

from textual.app import App, ComposeResult
from textual.widgets import Header, Footer, ListView, ListItem, Label, Markdown, Input, Button, TextArea, Select
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.screen import ModalScreen
from textual.reactive import reactive
from textual.theme import Theme

from .db import (
    init_db, add_snippet, get_snippet_fields, update_snippet,
    delete_snippet, search_snippets_full, add_tag, remove_tag,
    add_usage, get_usages, APP_DIR,
)
from .config import DEFAULT_CONFIG, load_config, save_config
import platform, os
if platform.system().lower() == "windows":
    os.system("")


ACCENT_COLORS = {
    "cyan": "#00ffff",
    "blue": "#3399ff",
    "green": "#4EBF71",
    "yellow": "#fabd2f",
    "orange": "#ff9e64",
    "pink": "#ff79c6",
    "purple": "#bd93f9",
    "red": "#ff5555",
}


def _sanitize_css_id(raw: str) -> str:
    """Turn an arbitrary snippet ID into a valid CSS identifier fragment.

    Textual widget IDs must be valid CSS identifiers.  Characters like
    dots, spaces, ``@``, etc. are replaced with underscores so that IDs
    such as ``ALG.000001`` become ``ALG_000001``.
    """
    return _re.sub(r"[^A-Za-z0-9_-]", "_", raw)



MODAL_BASE_CSS = """
.modal-dialog {
    padding: 1 2;
    width: 70;
    max-width: 90%;
    max-height: 90%;
    border: thick $background 80%;
    background: $surface;
}
.modal-dialog Label {
    margin-top: 1;
}
.modal-title {
    margin-top: 0;
    text-style: bold;
}
.modal-body {
    height: 1fr;
    min-height: 8;
}
.modal-actions {
    height: 3;
    min-height: 3;
    margin-top: 1;
}
.modal-actions Button {
    margin-right: 1;
}
.modal-compact {
    width: 60;
    height: auto;
    max-height: 85%;
}
.modal-wide {
    width: 80;
}
"""


# ---------------------------------------------------------------------------
# Modal Screens
# ---------------------------------------------------------------------------

class AddSnippetModal(ModalScreen[dict]):
    """Modal for adding a new snippet with all metadata fields."""
    CSS = MODAL_BASE_CSS + """
    AddSnippetModal {
        align: center middle;
    }
    #add-dialog #code-input {
        height: 12;
        min-height: 8;
    }
    """
    def __init__(
        self,
        code_language: str = "python",
        default_tags: str = "",
        id_formats: dict | None = None,
        default_id_format: str = "default",
    ) -> None:
        super().__init__()
        self._code_language = code_language
        self._default_tags = default_tags
        self._id_formats = id_formats or {"default": DEFAULT_CONFIG["snippets"]["id_formats"]["default"]}
        self._default_id_format = (
            default_id_format if default_id_format in self._id_formats else "default"
        )

    def _format_options(self) -> list[tuple[str, str]]:
        options = []
        for name in sorted(self._id_formats):
            config = self._id_formats[name]
            pattern = config.get("pattern")
            if not pattern:
                width = config.get("width", "auto")
                digits = "#" * int(width) if str(width).isdigit() else "#..."
                pattern = f"{config.get('prefix', 'CP')}{digits}"
            options.append((f"{name} ({pattern})", name))
        return options

    def compose(self) -> ComposeResult:
        with Vertical(id="add-dialog", classes="modal-dialog modal-wide"):
            yield Label("➕ Add New Snippet", id="add-title", classes="modal-title")
            with VerticalScroll(id="add-form-body", classes="modal-body"):
                yield Label("Title:")
                yield Input(id="title-input")
                yield Label("Description:")
                yield Input(id="desc-input")
                yield Label("Use Case:")
                yield Input(id="use-input")
                yield Label("Tags (comma separated):")
                yield Input(value=self._default_tags, id="tags-input")
                yield Label("ID Format:")
                yield Select(
                    self._format_options(),
                    value=self._default_id_format,
                    allow_blank=False,
                    id="id-format-select",
                )
                yield Label("Code:")
                yield TextArea(id="code-input", language=self._code_language)
            with Horizontal(classes="modal-actions"):
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "save-btn":
            title = self.query_one("#title-input", Input).value.strip()
            desc = self.query_one("#desc-input", Input).value.strip()
            use = self.query_one("#use-input", Input).value.strip()
            tags = self.query_one("#tags-input", Input).value.strip()
            id_format = str(self.query_one("#id-format-select", Select).value)
            code = self.query_one("#code-input", TextArea).text.strip()
            if title and code:
                self.dismiss({
                    "title": title,
                    "desc": desc,
                    "use": use,
                    "tags": tags,
                    "id_format": id_format,
                    "code": code,
                })
            else:
                self.notify("Title and Code are required", severity="error")
        else:
            self.dismiss(None)


class EditSnippetModal(ModalScreen[dict]):
    """Modal for editing an existing snippet, pre-filled with current values."""
    CSS = MODAL_BASE_CSS + """
    EditSnippetModal {
        align: center middle;
    }
    #edit-dialog #code-input {
        height: 12;
        min-height: 8;
    }
    """

    def __init__(self, snippet_id: str, title: str, desc: str,
                 use_case: str, tags: str, code: str,
                 code_language: str = "python") -> None:
        super().__init__()
        self._snippet_id = snippet_id
        self._title = title
        self._desc = desc
        self._use_case = use_case
        self._tags = tags
        self._code = code
        self._code_language = code_language

    def compose(self) -> ComposeResult:
        with Vertical(id="edit-dialog", classes="modal-dialog modal-wide"):
            yield Label(f"✏️  Edit Snippet {self._snippet_id}", id="edit-title", classes="modal-title")
            with VerticalScroll(id="edit-form-body", classes="modal-body"):
                yield Label("Title:")
                yield Input(value=self._title, id="title-input")
                yield Label("Description:")
                yield Input(value=self._desc, id="desc-input")
                yield Label("Use Case:")
                yield Input(value=self._use_case, id="use-input")
                yield Label("Tags (comma separated):")
                yield Input(value=self._tags, id="tags-input")
                yield Label("Code:")
                yield TextArea(id="code-input", language=self._code_language)
            with Horizontal(classes="modal-actions"):
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_mount(self) -> None:
        self.query_one("#code-input", TextArea).load_text(self._code)

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "save-btn":
            title = self.query_one("#title-input", Input).value.strip()
            desc = self.query_one("#desc-input", Input).value.strip()
            use = self.query_one("#use-input", Input).value.strip()
            tags = self.query_one("#tags-input", Input).value.strip()
            code = self.query_one("#code-input", TextArea).text.strip()
            if title and code:
                self.dismiss({"title": title, "desc": desc, "use": use, "tags": tags, "code": code})
            else:
                self.notify("Title and Code are required", severity="error")
        else:
            self.dismiss(None)


class ConfirmDeleteModal(ModalScreen[bool]):
    """Confirmation dialog before deleting a snippet."""
    CSS = MODAL_BASE_CSS + """
    ConfirmDeleteModal {
        align: center middle;
    }
    #confirm-dialog {
        border: thick $error;
    }
    #confirm-dialog Label {
        text-align: center;
        width: 100%;
    }
    """

    def __init__(self, snippet_id: str, title: str) -> None:
        super().__init__()
        self._snippet_id = snippet_id
        self._title = title

    def compose(self) -> ComposeResult:
        with Vertical(id="confirm-dialog", classes="modal-dialog modal-compact"):
            yield Label(f"🗑  Delete '{self._title}' ({self._snippet_id})?", classes="modal-title")
            yield Label("This action cannot be undone.")
            with Horizontal(classes="modal-actions"):
                yield Button("Delete", variant="error", id="confirm-btn")
                yield Button("Cancel", variant="primary", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        self.dismiss(event.button.id == "confirm-btn")


class EditTagsModal(ModalScreen[dict]):
    """Modal for adding or removing a tag from a snippet."""
    CSS = MODAL_BASE_CSS + """
    EditTagsModal {
        align: center middle;
    }
    """

    def __init__(self, current_tags: str = "") -> None:
        super().__init__()
        self._current_tags = current_tags

    def compose(self) -> ComposeResult:
        with Vertical(id="tags-dialog", classes="modal-dialog modal-compact"):
            yield Label("🏷  Edit Tags", classes="modal-title")
            yield Label("Current Tags:")
            yield Input(value=self._current_tags, id="current-tags-input", disabled=True)
            yield Label("Tag:")
            yield Input(placeholder="tag name", id="tag-input")
            with Horizontal(classes="modal-actions"):
                yield Button("Add", variant="success", id="add-btn")
                yield Button("Remove", variant="warning", id="remove-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id in {"add-btn", "remove-btn"}:
            tag = self.query_one("#tag-input", Input).value.strip()
            if tag:
                action = "add" if event.button.id == "add-btn" else "remove"
                self.dismiss({"action": action, "tag": tag})
            else:
                self.notify("Enter a tag first", severity="error")
        else:
            self.dismiss(None)


class UseSnippetModal(ModalScreen[dict]):
    """Modal for recording snippet usage."""
    CSS = MODAL_BASE_CSS + """
    UseSnippetModal {
        align: center middle;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(id="use-dialog", classes="modal-dialog modal-compact"):
            yield Label("📝 Record Usage", classes="modal-title")
            yield Label("File path where used:")
            yield Input(id="file-input")
            with Horizontal(classes="modal-actions"):
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "save-btn":
            file_path = self.query_one("#file-input", Input).value.strip()
            if file_path:
                self.dismiss({"file": file_path})
            else:
                self.notify("File path is required", severity="error")
        else:
            self.dismiss(None)


class SettingsModal(ModalScreen[dict]):
    """Modal for updating persistent display settings."""
    CSS = MODAL_BASE_CSS + """
    SettingsModal {
        align: center middle;
    }
    #settings-dialog {
        width: 62;
        border: thick $primary;
    }
    """

    def __init__(self, themes: list[str], current_theme: str, current_accent: str, id_formats: dict, layout: str = "horizontal", border_style: str = "solid") -> None:
        super().__init__()
        self._themes = themes
        self._current_theme = current_theme
        self._current_accent = current_accent
        self._id_formats = id_formats
        self._layout = layout
        self._border_style = border_style

    def compose(self) -> ComposeResult:
        with Vertical(id="settings-dialog", classes="modal-dialog modal-compact"):
            yield Label("Settings", classes="modal-title")
            yield Label("Theme:")
            yield Select(
                [(theme, theme) for theme in self._themes],
                value=self._current_theme,
                allow_blank=False,
                id="theme-select",
            )
            yield Label("Accent:")
            yield Select(
                [(name.title(), name) for name in ACCENT_COLORS],
                value=self._current_accent,
                allow_blank=False,
                id="accent-select",
            )
            yield Label("Layout:")
            yield Select(
                [("Horizontal", "horizontal"), ("Vertical", "vertical")],
                value=self._layout,
                allow_blank=False,
                id="layout-select",
            )
            yield Label("Border Style:")
            yield Select(
                [(b.title(), b) for b in ["solid", "heavy", "rounded", "double"]],
                value=self._border_style,
                allow_blank=False,
                id="border-select",
            )
            yield Label("Format Colors:")
            for fmt_name, fmt_cfg in self._id_formats.items():
                current_color = fmt_cfg.get("color", "cyan")
                safe_name = _sanitize_css_id(fmt_name)
                yield Select(
                    [(name.title(), name) for name in ACCENT_COLORS],
                    value=current_color,
                    allow_blank=False,
                    id=f"fmt-color-{safe_name}",
                    prompt=f"{fmt_name} color",
                )
            with Horizontal(classes="modal-actions"):
                yield Button("Apply", variant="success", id="apply-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "apply-btn":
            format_colors = {}
            for fmt_name in self._id_formats:
                safe_name = _sanitize_css_id(fmt_name)
                format_colors[fmt_name] = str(self.query_one(f"#fmt-color-{safe_name}", Select).value)
            
            self.dismiss({
                "theme": str(self.query_one("#theme-select", Select).value),
                "accent_color": str(self.query_one("#accent-select", Select).value),
                "layout": str(self.query_one("#layout-select", Select).value),
                "border_style": str(self.query_one("#border-select", Select).value),
                "format_colors": format_colors,
            })
        else:
            self.dismiss(None)


# ---------------------------------------------------------------------------
# Main Application
# ---------------------------------------------------------------------------

class SnippetApp(App):
    """A Textual App to browse CPKB Snippets."""

    TITLE = "CPKB - Competitive Programming Knowledge Base"

    CSS = """
    #left-pane {
        border-right: solid $primary;
        height: 100%;
    }
    #right-pane {
        padding: 1 2;
        height: 100%;
        overflow-y: auto;
    }
    ListItem {
        padding: 1;
    }
    """

    left_pane_width: reactive[int] = reactive(35)

    _DEFAULT_BINDINGS = [
        ("quit", "ctrl+q", "Quit", True),
        ("refresh", "ctrl+r", "Refresh List", True),
        ("copy_snippet", "ctrl+c", "Copy Code", True),
        ("focus_search", "/", "Search", True),
        ("add_snippet", "ctrl+a", "Add", True),
        ("edit_snippet", "ctrl+e", "Edit", True),
        ("use_snippet", "ctrl+u", "Use", True),
        ("delete_snippet", "ctrl+d", "Delete", True),
        ("edit_tags", "ctrl+t", "Edit Tags", True),
        ("settings", "ctrl+comma", "Settings", True),
        ("scroll_detail_down", "j", "Detail Down", False),
        ("scroll_detail_up", "k", "Detail Up", False),
        ("page_detail_down", "pagedown", "Detail Page Down", False),
        ("page_detail_up", "pageup", "Detail Page Up", False),
        ("shrink_left", "left_square_bracket", "Shrink Left", False),
        ("grow_left", "right_square_bracket", "Grow Left", False),
    ]

    BINDINGS = []

    def compose(self) -> ComposeResult:
        yield Header()
        # the container class will be updated in _apply_display_config using CSS, so we just use Horizontal here
        with Horizontal(id="main-container"):
            with Vertical(id="left-pane"):
                yield Input(placeholder="Search snippets (/)", id="search-input")
                yield ListView(id="snippet-list")
            with Vertical(id="right-pane"):
                yield Markdown("Select a snippet from the list to view its contents.", id="snippet-view")
        yield Footer()

    def _load_display_config(self) -> tuple[str, str]:
        config = load_config(APP_DIR)
        display = config.get("display", {})
        theme = str(display.get("theme", DEFAULT_CONFIG["display"]["theme"]))
        accent = str(display.get("accent_color", DEFAULT_CONFIG["display"]["accent_color"]))
        try:
            pane_width = int(display.get("left_pane_width", DEFAULT_CONFIG["display"]["left_pane_width"]))
        except (TypeError, ValueError):
            pane_width = DEFAULT_CONFIG["display"]["left_pane_width"]
        self.layout_dir = str(display.get("layout", DEFAULT_CONFIG["display"]["layout"]))
        self.border_style = str(display.get("border_style", DEFAULT_CONFIG["display"]["border_style"]))
        
        self.left_pane_width = max(
            15,
            min(70, pane_width),
        )
        snippets_config = config.get("snippets", {})
        self.code_language = str(
            snippets_config.get("code_language")
            or config.get("default_language")
            or DEFAULT_CONFIG["snippets"]["code_language"]
        )
        self.default_tags = str(snippets_config.get("default_tags") or "")
        self.id_formats = snippets_config.get("id_formats", DEFAULT_CONFIG["snippets"]["id_formats"])
        
        # Ensure default colors for formats
        color_names = list(ACCENT_COLORS.keys())
        for i, (fmt_name, fmt_cfg) in enumerate(self.id_formats.items()):
            if "color" not in fmt_cfg:
                fmt_cfg["color"] = color_names[i % len(color_names)]

        self.default_id_format = str(snippets_config.get("default_id_format", "default"))
        return theme, accent

    def _custom_theme_name(self, theme: str, accent: str) -> str:
        return f"cpkb-{theme}-{accent}"

    def _apply_display_config(self, theme: str, accent: str) -> tuple[str, str]:
        if theme not in self.available_themes:
            theme = DEFAULT_CONFIG["display"]["theme"]
        if accent not in ACCENT_COLORS:
            accent = DEFAULT_CONFIG["display"]["accent_color"]

        base = self.available_themes[theme]
        custom_theme = Theme(
            name=self._custom_theme_name(theme, accent),
            primary=ACCENT_COLORS[accent],
            secondary=base.secondary,
            warning=base.warning,
            error=base.error,
            success=base.success,
            accent=ACCENT_COLORS[accent],
            foreground=base.foreground,
            background=base.background,
            surface=base.surface,
            panel=base.panel,
            boost=base.boost,
            dark=base.dark,
            luminosity_spread=base.luminosity_spread,
            text_alpha=base.text_alpha,
            variables=dict(base.variables),
            ansi=base.ansi,
        )
        if custom_theme.name in self.available_themes:
            self.unregister_theme(custom_theme.name)
        self.register_theme(custom_theme)
        self.theme = custom_theme.name
        self.display_theme = theme
        self.display_accent = accent
        
        # update layout styles dynamically
        try:
            main_container = self.query_one("#main-container")
            left_pane = self.query_one("#left-pane")
            right_pane = self.query_one("#right-pane")
            
            if self.layout_dir == "vertical":
                main_container.styles.layout = "vertical"
                left_pane.styles.width = "100%"
                left_pane.styles.height = f"{self.left_pane_width}%"
                left_pane.styles.border_right = "none"
                left_pane.styles.border_bottom = (self.border_style, custom_theme.primary)
                right_pane.styles.width = "100%"
                right_pane.styles.height = f"{100 - self.left_pane_width}%"
            else:
                main_container.styles.layout = "horizontal"
                left_pane.styles.height = "100%"
                left_pane.styles.width = f"{self.left_pane_width}%"
                left_pane.styles.border_bottom = "none"
                left_pane.styles.border_right = (self.border_style, custom_theme.primary)
                right_pane.styles.height = "100%"
                right_pane.styles.width = f"{100 - self.left_pane_width}%"
        except Exception:
            pass # before mount
            
        return theme, accent

    def _save_display_config(self, theme: str, accent: str, layout: str = "horizontal", border_style: str = "solid") -> None:
        config = load_config(APP_DIR)
        display = config.setdefault("display", {})
        display["theme"] = theme
        display["accent_color"] = accent
        display["layout"] = layout
        display["border_style"] = border_style
        save_config(APP_DIR, config)

    def watch_left_pane_width(self, value: int) -> None:
        """Update pane widths/heights when the reactive property changes."""
        try:
            if self.layout_dir == "vertical":
                self.query_one("#left-pane").styles.height = f"{value}%"
                self.query_one("#right-pane").styles.height = f"{100 - value}%"
            else:
                self.query_one("#left-pane").styles.width = f"{value}%"
                self.query_one("#right-pane").styles.width = f"{100 - value}%"
        except Exception:
            pass  # widgets not yet mounted

    def action_shrink_left(self) -> None:
        """Shrink the left pane by 5%."""
        self.left_pane_width = max(15, self.left_pane_width - 5)

    def action_grow_left(self) -> None:
        """Grow the left pane by 5%."""
        self.left_pane_width = min(70, self.left_pane_width + 5)

    def action_scroll_detail_down(self) -> None:
        """Scroll the snippet detail pane down."""
        self.query_one("#snippet-view", Markdown).scroll_down(animate=False)

    def action_scroll_detail_up(self) -> None:
        """Scroll the snippet detail pane up."""
        self.query_one("#snippet-view", Markdown).scroll_up(animate=False)

    def action_page_detail_down(self) -> None:
        """Page the snippet detail pane down."""
        self.query_one("#snippet-view", Markdown).scroll_page_down(animate=False)

    def action_page_detail_up(self) -> None:
        """Page the snippet detail pane up."""
        self.query_one("#snippet-view", Markdown).scroll_page_up(animate=False)

    async def on_mount(self) -> None:
        self.conn = init_db()
        self.cursor = self.conn.cursor()
        self._widget_id_to_snippet: dict[str, str] = {}  # sanitized widget-id -> real snippet id
        theme, accent = self._load_display_config()
        self._apply_display_config(theme, accent)
        self.watch_left_pane_width(self.left_pane_width)
        
        # Load custom keybindings
        config = load_config(APP_DIR)
        kb = config.get("keybindings", DEFAULT_CONFIG["keybindings"])
        for action, default_key, desc, show in self._DEFAULT_BINDINGS:
            key = kb.get(action, default_key)
            self.bind(key, action, description=desc, show=show)
            
        await self.action_refresh()

    def action_focus_search(self) -> None:
        self.query_one("#search-input", Input).focus()

    async def on_input_changed(self, event: Input.Changed) -> None:
        if event.input.id == "search-input":
            await self.action_refresh(query=event.value)

    async def action_settings(self) -> None:
        """Open persistent display settings."""
        themes = sorted(
            theme for theme in self.available_themes
            if not theme.startswith("cpkb-")
        )

        def check_result(result: dict | None) -> None:
            if result:
                self.layout_dir = result["layout"]
                self.border_style = result["border_style"]
                theme, accent = self._apply_display_config(result["theme"], result["accent_color"])
                self._save_display_config(theme, accent, result["layout"], result["border_style"])
                
                # Save format colors
                config = load_config(APP_DIR)
                snippets_cfg = config.setdefault("snippets", {})
                id_formats = snippets_cfg.setdefault("id_formats", DEFAULT_CONFIG["snippets"]["id_formats"])
                for fmt_name, color in result["format_colors"].items():
                    if fmt_name in id_formats:
                        id_formats[fmt_name]["color"] = color
                save_config(APP_DIR, config)
                self.id_formats = id_formats
                self.run_worker(self.action_refresh())
                
                self.notify("Display settings saved.")

        self.push_screen(
            SettingsModal(themes, self.display_theme, self.display_accent, self.id_formats, self.layout_dir, self.border_style),
            check_result,
        )

    # ------------------------------------------------------------------
    # Refresh
    # ------------------------------------------------------------------

    def _get_format_color(self, snippet_id: str) -> str:
        for fmt_name, fmt_cfg in self.id_formats.items():
            pattern = fmt_cfg.get("pattern")
            if pattern:
                prefix = pattern.split("<")[0].split("#")[0]
            else:
                prefix = fmt_cfg.get("prefix", "CP")
            if snippet_id.startswith(prefix):
                return fmt_cfg.get("color", "cyan")
        return "white"

    async def action_refresh(self, query: str = "") -> None:
        """Refresh the list of snippets."""
        list_view = self.query_one("#snippet-list", ListView)
        await list_view.clear()

        if query:
            rows = search_snippets_full(self.cursor, query)
        else:
            self.cursor.execute("SELECT id, title FROM snippets ORDER BY created_at DESC")
            rows = self.cursor.fetchall()

        self._widget_id_to_snippet = {}
        for row in rows:
            snippet_id = row[0]
            widget_id = f"item_{_sanitize_css_id(snippet_id)}"
            self._widget_id_to_snippet[widget_id] = snippet_id
            color = self._get_format_color(snippet_id)
            if color in ACCENT_COLORS:
                color_hex = ACCENT_COLORS[color]
            else:
                color_hex = color
            list_view.append(ListItem(Label(f"[{color_hex}]{row[0]}[/] - {row[1]}"), id=widget_id))

        if rows:
            list_view.index = 0
        else:
            self.query_one("#snippet-view", Markdown).update("No snippets found matching your search.")

    # ------------------------------------------------------------------
    # Copy
    # ------------------------------------------------------------------

    def action_copy_snippet(self) -> None:
        """Copy the code of the currently selected snippet to the clipboard."""
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = self._widget_id_to_snippet.get(list_view.highlighted_child.id, "")
            row = get_snippet_fields(self.cursor, snippet_id, "code") if snippet_id else None
            if row:
                self.copy_to_clipboard(row[0])
                self.notify(f"Code for {snippet_id} copied to clipboard!", title="Copied!")

    # ------------------------------------------------------------------
    # Edit (native modal)
    # ------------------------------------------------------------------

    async def action_edit_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = self._widget_id_to_snippet.get(list_view.highlighted_child.id, "")
            if not snippet_id:
                return
            row = get_snippet_fields(self.cursor, snippet_id)
            if not row:
                self.notify(f"Snippet {snippet_id} not found", severity="error")
                return

            title, desc, use_case, tags, code = row

            def check_result(result: dict | None) -> None:
                if result:
                    update_snippet(self.cursor, self.conn, snippet_id,
                                   result["title"], result["desc"],
                                   result["use"], result["tags"], result["code"])
                    self.run_worker(self.action_refresh())
                    self.notify(f"Snippet {snippet_id} updated!")

            self.push_screen(
                EditSnippetModal(
                    snippet_id,
                    title,
                    desc or "",
                    use_case or "",
                    tags or "",
                    code,
                    self.code_language,
                ),
                check_result,
            )

    # ------------------------------------------------------------------
    # Delete (native modal)
    # ------------------------------------------------------------------

    async def action_delete_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = self._widget_id_to_snippet.get(list_view.highlighted_child.id, "")
            if not snippet_id:
                return
            row = get_snippet_fields(self.cursor, snippet_id, "id, title")
            if not row:
                self.notify(f"Snippet {snippet_id} not found", severity="error")
                return

            def check_result(confirmed: bool) -> None:
                if confirmed:
                    delete_snippet(self.cursor, self.conn, snippet_id)
                    self.run_worker(self.action_refresh())
                    self.notify(f"Snippet {snippet_id} deleted!")

            self.push_screen(ConfirmDeleteModal(snippet_id, row[1]), check_result)

    # ------------------------------------------------------------------
    # Tags (native modal)
    # ------------------------------------------------------------------

    async def action_edit_tags(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = self._widget_id_to_snippet.get(list_view.highlighted_child.id, "")
            if not snippet_id:
                return
            row = get_snippet_fields(self.cursor, snippet_id, "tags")
            current_tags = row[0] if row and row[0] else ""

            def check_result(result: dict | None) -> None:
                if result:
                    try:
                        if result["action"] == "add":
                            add_tag(self.cursor, self.conn, snippet_id, result["tag"])
                        else:
                            remove_tag(self.cursor, self.conn, snippet_id, result["tag"])
                        self.run_worker(self.action_refresh())
                        self.notify(f"Tags updated for {snippet_id}!")
                    except ValueError as e:
                        self.notify(str(e), severity="error")

            self.push_screen(EditTagsModal(current_tags), check_result)

    # ------------------------------------------------------------------
    # Add (native modal)
    # ------------------------------------------------------------------

    async def action_add_snippet(self) -> None:
        def check_result(result: dict | None) -> None:
            if result:
                snippet_id = add_snippet(
                    self.cursor, self.conn,
                    result["title"], result["desc"], result["use"],
                    result["tags"], result["code"], result["id_format"],
                )
                self.run_worker(self.action_refresh())
                self.notify(f"Added snippet {snippet_id}!")

        self.push_screen(
            AddSnippetModal(
                self.code_language,
                self.default_tags,
                self.id_formats,
                self.default_id_format,
            ),
            check_result,
        )

    # ------------------------------------------------------------------
    # Use (native modal)
    # ------------------------------------------------------------------

    async def action_use_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = self._widget_id_to_snippet.get(list_view.highlighted_child.id, "")
            if not snippet_id:
                return

            def check_result(result: dict | None) -> None:
                if result:
                    add_usage(self.cursor, self.conn, snippet_id, result["file"])
                    self.run_worker(self.action_refresh())
                    self.notify(f"Usage recorded for {snippet_id}!")

            self.push_screen(UseSnippetModal(), check_result)

    # ------------------------------------------------------------------
    # Detail View
    # ------------------------------------------------------------------

    def on_list_view_selected(self, event: ListView.Selected) -> None:
        if not event.item.id:
            return

        snippet_id = self._widget_id_to_snippet.get(event.item.id, "")
        if not snippet_id:
            return
        row = get_snippet_fields(self.cursor, snippet_id)

        if row:
            md_content = f"# {row[0]}\n\n"
            if row[1]: md_content += f"**Description:** {row[1]}\n\n"
            if row[2]: md_content += f"**Use Case:** {row[2]}\n\n"
            if row[3]: md_content += f"**Tags:** {row[3]}\n\n"
            md_content += f"```{self.code_language}\n{row[4]}\n```\n"

            # Fetch usages
            usages = get_usages(self.cursor, snippet_id)
            if usages:
                md_content += "\n### Usages\n"
                for u in usages:
                    date_str = u[4][:10]
                    prob = f" ({u[2]})" if u[2] else ""
                    md_content += f"- {date_str}: `{u[1]}`{prob}\n"

            self.query_one("#snippet-view", Markdown).update(md_content)


def run_tui():
    app = SnippetApp()
    app.run()
