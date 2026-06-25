from textual.app import App, ComposeResult
from textual.widgets import Header, Footer, ListView, ListItem, Label, Markdown, Input, Button, TextArea
from textual.containers import Horizontal, Vertical
from textual.binding import Binding
from textual.screen import ModalScreen

from .db import (
    init_db, add_snippet, get_snippet_fields, update_snippet,
    delete_snippet, search_snippets_full, add_tag, remove_tag,
    add_usage, get_usages,
)
import platform, os
if platform.system().lower() == "windows":
    os.system("")


# ---------------------------------------------------------------------------
# Modal Screens
# ---------------------------------------------------------------------------

class AddSnippetModal(ModalScreen[dict]):
    """Modal for adding a new snippet with all metadata fields."""
    CSS = """
    AddSnippetModal {
        align: center middle;
    }
    #add-dialog {
        padding: 1 2;
        width: 70;
        height: 32;
        border: thick $background 80%;
        background: $surface;
    }
    #add-dialog Label {
        margin-top: 1;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(id="add-dialog"):
            yield Label("➕ Add New Snippet", id="add-title")
            yield Label("Title:")
            yield Input(id="title-input")
            yield Label("Description:")
            yield Input(id="desc-input")
            yield Label("Use Case:")
            yield Input(id="use-input")
            yield Label("Tags (comma separated):")
            yield Input(id="tags-input")
            yield Label("Code:")
            yield TextArea(id="code-input", language="python")
            with Horizontal():
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

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


class EditSnippetModal(ModalScreen[dict]):
    """Modal for editing an existing snippet, pre-filled with current values."""
    CSS = """
    EditSnippetModal {
        align: center middle;
    }
    #edit-dialog {
        padding: 1 2;
        width: 70;
        height: 32;
        border: thick $background 80%;
        background: $surface;
    }
    #edit-dialog Label {
        margin-top: 1;
    }
    """

    def __init__(self, snippet_id: str, title: str, desc: str,
                 use_case: str, tags: str, code: str) -> None:
        super().__init__()
        self._snippet_id = snippet_id
        self._title = title
        self._desc = desc
        self._use_case = use_case
        self._tags = tags
        self._code = code

    def compose(self) -> ComposeResult:
        with Vertical(id="edit-dialog"):
            yield Label(f"✏️  Edit Snippet {self._snippet_id}", id="edit-title")
            yield Label("Title:")
            yield Input(value=self._title, id="title-input")
            yield Label("Description:")
            yield Input(value=self._desc, id="desc-input")
            yield Label("Use Case:")
            yield Input(value=self._use_case, id="use-input")
            yield Label("Tags (comma separated):")
            yield Input(value=self._tags, id="tags-input")
            yield Label("Code:")
            yield TextArea(id="code-input", language="python")
            with Horizontal():
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
    CSS = """
    ConfirmDeleteModal {
        align: center middle;
    }
    #confirm-dialog {
        padding: 2 4;
        width: 50;
        height: 10;
        border: thick $error;
        background: $surface;
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
        with Vertical(id="confirm-dialog"):
            yield Label(f"🗑  Delete '{self._title}' ({self._snippet_id})?")
            yield Label("This action cannot be undone.")
            with Horizontal():
                yield Button("Delete", variant="error", id="confirm-btn")
                yield Button("Cancel", variant="primary", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        self.dismiss(event.button.id == "confirm-btn")


class EditTagsModal(ModalScreen[dict]):
    """Modal for adding or removing a tag from a snippet."""
    CSS = """
    EditTagsModal {
        align: center middle;
    }
    #tags-dialog {
        padding: 1 2;
        width: 45;
        height: 14;
        border: thick $background 80%;
        background: $surface;
    }
    #tags-dialog Label {
        margin-top: 1;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(id="tags-dialog"):
            yield Label("🏷  Edit Tags")
            yield Label("Action:")
            yield Input(placeholder="add or remove", id="action-input")
            yield Label("Tag:")
            yield Input(id="tag-input")
            with Horizontal():
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "save-btn":
            action = self.query_one("#action-input", Input).value.strip().lower()
            tag = self.query_one("#tag-input", Input).value.strip()
            if action in ('a', 'add', 'r', 'remove') and tag:
                self.dismiss({"action": "add" if action.startswith('a') else "remove", "tag": tag})
            else:
                self.notify("Enter a valid action (add/remove) and tag", severity="error")
        else:
            self.dismiss(None)


class UseSnippetModal(ModalScreen[dict]):
    """Modal for recording snippet usage."""
    CSS = """
    UseSnippetModal {
        align: center middle;
    }
    #use-dialog {
        padding: 1 2;
        width: 55;
        height: 12;
        border: thick $background 80%;
        background: $surface;
    }
    #use-dialog Label {
        margin-top: 1;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(id="use-dialog"):
            yield Label("📝 Record Usage")
            yield Label("File path where used:")
            yield Input(id="file-input")
            with Horizontal():
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


# ---------------------------------------------------------------------------
# Main Application
# ---------------------------------------------------------------------------

class SnippetApp(App):
    """A Textual App to browse CPKB Snippets."""

    TITLE = "CPKB - Competitive Programming Knowledge Base"

    CSS = """
    #left-pane {
        width: 35%;
        border-right: solid $primary;
        height: 100%;
    }
    #right-pane {
        width: 65%;
        padding: 1 2;
        height: 100%;
        overflow-y: auto;
    }
    ListItem {
        padding: 1;
    }
    """

    BINDINGS = [
        Binding("ctrl+q", "quit", "Quit"),
        Binding("ctrl+r", "refresh", "Refresh List"),
        Binding("ctrl+c", "copy_snippet", "Copy Code"),
        Binding("/", "focus_search", "Search"),
        Binding("ctrl+a", "add_snippet", "Add"),
        Binding("ctrl+e", "edit_snippet", "Edit"),
        Binding("ctrl+u", "use_snippet", "Use"),
        Binding("ctrl+d", "delete_snippet", "Delete"),
        Binding("ctrl+t", "edit_tags", "Edit Tags"),
    ]

    def compose(self) -> ComposeResult:
        yield Header()
        with Horizontal():
            with Vertical(id="left-pane"):
                yield Input(placeholder="Search snippets (/)", id="search-input")
                yield ListView(id="snippet-list")
            with Vertical(id="right-pane"):
                yield Markdown("Select a snippet from the list to view its contents.", id="snippet-view")
        yield Footer()

    async def on_mount(self) -> None:
        self.conn = init_db()
        self.cursor = self.conn.cursor()
        await self.action_refresh()

    def action_focus_search(self) -> None:
        self.query_one("#search-input", Input).focus()

    async def on_input_changed(self, event: Input.Changed) -> None:
        if event.input.id == "search-input":
            await self.action_refresh(query=event.value)

    # ------------------------------------------------------------------
    # Refresh
    # ------------------------------------------------------------------

    async def action_refresh(self, query: str = "") -> None:
        """Refresh the list of snippets."""
        list_view = self.query_one("#snippet-list", ListView)
        await list_view.clear()

        if query:
            rows = search_snippets_full(self.cursor, query)
        else:
            self.cursor.execute("SELECT id, title FROM snippets ORDER BY created_at DESC")
            rows = self.cursor.fetchall()

        for row in rows:
            list_view.append(ListItem(Label(f"{row[0]} - {row[1]}"), id=f"item_{row[0]}"))

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
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
            row = get_snippet_fields(self.cursor, snippet_id, "code")
            if row:
                self.copy_to_clipboard(row[0])
                self.notify(f"Code for {snippet_id} copied to clipboard!", title="Copied!")

    # ------------------------------------------------------------------
    # Edit (native modal)
    # ------------------------------------------------------------------

    async def action_edit_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
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
                EditSnippetModal(snippet_id, title, desc or "", use_case or "", tags or "", code),
                check_result,
            )

    # ------------------------------------------------------------------
    # Delete (native modal)
    # ------------------------------------------------------------------

    async def action_delete_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
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
            snippet_id = list_view.highlighted_child.id.replace("item_", "")

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

            self.push_screen(EditTagsModal(), check_result)

    # ------------------------------------------------------------------
    # Add (native modal)
    # ------------------------------------------------------------------

    async def action_add_snippet(self) -> None:
        def check_result(result: dict | None) -> None:
            if result:
                snippet_id = add_snippet(
                    self.cursor, self.conn,
                    result["title"], result["desc"], result["use"],
                    result["tags"], result["code"],
                )
                self.run_worker(self.action_refresh())
                self.notify(f"Added snippet {snippet_id}!")

        self.push_screen(AddSnippetModal(), check_result)

    # ------------------------------------------------------------------
    # Use (native modal)
    # ------------------------------------------------------------------

    async def action_use_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")

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

        snippet_id = event.item.id.replace("item_", "")
        row = get_snippet_fields(self.cursor, snippet_id)

        if row:
            md_content = f"# {row[0]}\n\n"
            if row[1]: md_content += f"**Description:** {row[1]}\n\n"
            if row[2]: md_content += f"**Use Case:** {row[2]}\n\n"
            if row[3]: md_content += f"**Tags:** {row[3]}\n\n"
            md_content += f"```python\n{row[4]}\n```\n"

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
