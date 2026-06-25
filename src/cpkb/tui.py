from textual.app import App, ComposeResult
from textual.widgets import Header, Footer, ListView, ListItem, Label, Markdown, Input
from textual.containers import Horizontal, Vertical
from textual.binding import Binding

# We can import init_db from cli since they are in the same package
from .cli import init_db
import platform, os
if platform.system().lower() == "windows":
    os.system("")

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

    async def action_refresh(self, query: str = "") -> None:
        """Refresh the list of snippets."""
        list_view = self.query_one("#snippet-list", ListView)
        await list_view.clear()
        
        if query:
            query_parts = query.lower().split()
            conditions = []
            params = []
            for part in query_parts:
                like_str = f"%{part}%"
                conditions.append("(LOWER(title) LIKE ? OR LOWER(description) LIKE ? OR LOWER(tags) LIKE ? OR LOWER(code) LIKE ?)")
                params.extend([like_str, like_str, like_str, like_str])
            
            query_sql = "SELECT id, title FROM snippets WHERE " + " AND ".join(conditions) + " ORDER BY created_at DESC"
            self.cursor.execute(query_sql, params)
        else:
            self.cursor.execute("SELECT id, title FROM snippets ORDER BY created_at DESC")
            
        rows = self.cursor.fetchall()
        for row in rows:
            list_view.append(ListItem(Label(f"{row[0]} - {row[1]}"), id=f"item_{row[0]}"))
            
        if rows:
            list_view.index = 0
        else:
            self.query_one("#snippet-view", Markdown).update("No snippets found matching your search.")

    def action_copy_snippet(self) -> None:
        """Copy the code of the currently selected snippet to the clipboard."""
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
            self.cursor.execute("SELECT code FROM snippets WHERE id = ?", (snippet_id,))
            row = self.cursor.fetchone()
            if row:
                self.copy_to_clipboard(row[0])
                self.notify(f"Code for {snippet_id} copied to clipboard!", title="Copied!")

    async def action_edit_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
            
            with self.suspend():
                from .cli import cmd_edit
                class DummyArgs: id = snippet_id
                cmd_edit(DummyArgs())
                
            await self.action_refresh()
            self.notify(f"Snippet {snippet_id} updated.")

    async def action_delete_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
            
            with self.suspend():
                from .cli import cmd_delete
                class DummyArgs: id = snippet_id
                cmd_delete(DummyArgs())
                
            await self.action_refresh()

from textual.screen import ModalScreen
from textual.widgets import Button, TextArea

class AddSnippetModal(ModalScreen[dict]):
    CSS = """
    AddSnippetModal {
        align: center middle;
    }
    #add-dialog {
        padding: 1 2;
        width: 60;
        height: 30;
        border: thick $background 80%;
        background: $surface;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(id="add-dialog"):
            yield Label("Title:")
            yield Input(id="title-input")
            yield Label("Description:")
            yield Input(id="desc-input")
            yield Label("Use Case:")
            yield Input(id="use-input")
            yield Label("Tags:")
            yield Input(id="tags-input")
            yield Label("Code:")
            yield TextArea(id="code-input", language="python")
            with Horizontal():
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "save-btn":
            title = self.query_one("#title-input", Input).value
            desc = self.query_one("#desc-input", Input).value
            use = self.query_one("#use-input", Input).value
            tags = self.query_one("#tags-input", Input).value
            code = self.query_one("#code-input", TextArea).text
            if title and code:
                self.dismiss({"title": title, "desc": desc, "use": use, "tags": tags, "code": code})
            else:
                self.notify("Title and Code are required", severity="error")
        else:
            self.dismiss(None)

class EditTagsModal(ModalScreen[dict]):
    CSS = """
    EditTagsModal {
        align: center middle;
    }
    #tags-dialog {
        padding: 1 2;
        width: 40;
        height: 15;
        border: thick $background 80%;
        background: $surface;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(id="tags-dialog"):
            yield Label("Action (add/remove):")
            yield Input(placeholder="add or remove", id="action-input")
            yield Label("Tag:")
            yield Input(id="tag-input")
            with Horizontal():
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "save-btn":
            action = self.query_one("#action-input", Input).value.lower()
            tag = self.query_one("#tag-input", Input).value
            if action in ['a', 'add', 'r', 'remove'] and tag:
                self.dismiss({"action": "add" if action.startswith('a') else "remove", "tag": tag})
            else:
                self.notify("Invalid action or empty tag", severity="error")
        else:
            self.dismiss(None)

class UseSnippetModal(ModalScreen[dict]):
    CSS = """
    UseSnippetModal {
        align: center middle;
    }
    #use-dialog {
        padding: 1 2;
        width: 50;
        height: 15;
        border: thick $background 80%;
        background: $surface;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(id="use-dialog"):
            yield Label("File path where used:")
            yield Input(id="file-input")
            with Horizontal():
                yield Button("Save", variant="success", id="save-btn")
                yield Button("Cancel", variant="error", id="cancel-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "save-btn":
            file_path = self.query_one("#file-input", Input).value
            if file_path:
                self.dismiss({"file": file_path})
            else:
                self.notify("File path is required", severity="error")
        else:
            self.dismiss(None)

    async def action_edit_tags(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
            
            def check_result(result: dict | None) -> None:
                if result:
                    if result["action"] == "add":
                        from .cli import cmd_tag_add
                        class DummyArgs:
                            id = snippet_id
                            tag = result["tag"]
                        cmd_tag_add(DummyArgs())
                    elif result["action"] == "remove":
                        from .cli import cmd_tag_remove
                        class DummyArgs:
                            id = snippet_id
                            tag = result["tag"]
                        cmd_tag_remove(DummyArgs())
                    self.run_worker(self.action_refresh())
                    self.notify(f"Tags updated for {snippet_id}!")
            
            self.push_screen(EditTagsModal(), check_result)

    async def action_add_snippet(self) -> None:
        def check_result(result: dict | None) -> None:
            if result:
                conn = init_db()
                cursor = conn.cursor()
                from .cli import generate_id, update_tags
                from datetime import datetime
                
                snippet_id = generate_id(cursor)
                now = datetime.utcnow().isoformat()
                
                cursor.execute('''
                    INSERT INTO snippets (id, title, description, use_case, tags, code, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ''', (snippet_id, result["title"], result["desc"], result["use"], result["tags"], result["code"], now, now))
                
                update_tags(cursor, snippet_id, result["tags"])
                conn.commit()
                
                self.run_worker(self.action_refresh())
                self.notify(f"Added snippet {snippet_id}!")
                
        self.push_screen(AddSnippetModal(), check_result)

    async def action_use_snippet(self) -> None:
        list_view = self.query_one("#snippet-list", ListView)
        if list_view.highlighted_child and list_view.highlighted_child.id:
            snippet_id = list_view.highlighted_child.id.replace("item_", "")
            
            def check_result(result: dict | None) -> None:
                if result:
                    from .cli import cmd_use
                    class DummyArgs: 
                        id = snippet_id
                        file = result["file"]
                    cmd_use(DummyArgs())
                    self.run_worker(self.action_refresh())
                    self.notify(f"Usage recorded for {snippet_id}!")
                    
            self.push_screen(UseSnippetModal(), check_result)

    def on_list_view_selected(self, event: ListView.Selected) -> None:
        if not event.item.id:
            return
            
        snippet_id = event.item.id.replace("item_", "")
        self.cursor.execute("SELECT title, description, use_case, tags, code FROM snippets WHERE id = ?", (snippet_id,))
        row = self.cursor.fetchone()
        
        if row:
            md_content = f"# {row[0]}\n\n"
            if row[1]: md_content += f"**Description:** {row[1]}\n\n"
            if row[2]: md_content += f"**Use Case:** {row[2]}\n\n"
            if row[3]: md_content += f"**Tags:** {row[3]}\n\n"
            md_content += f"```python\n{row[4]}\n```\n"
            
            # Fetch usages
            self.cursor.execute("SELECT file_path, problem_name, created_at FROM usages WHERE snippet_id = ? ORDER BY created_at DESC", (snippet_id,))
            usages = self.cursor.fetchall()
            if usages:
                md_content += "\n### Usages\n"
                for u in usages:
                    date_str = u[2][:10]
                    prob = f" ({u[1]})" if u[1] else ""
                    md_content += f"- {date_str}: `{u[0]}`{prob}\n"
                    
            self.query_one("#snippet-view", Markdown).update(md_content)

def run_tui():
    app = SnippetApp()
    app.run()
