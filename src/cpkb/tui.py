from textual.app import App, ComposeResult
from textual.widgets import Header, Footer, ListView, ListItem, Label, Markdown
from textual.containers import Horizontal, Vertical
from textual.binding import Binding

# We can import init_db from cli since they are in the same package
from .cli import init_db

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
        Binding("q", "quit", "Quit"),
        Binding("r", "refresh", "Refresh List"),
        Binding("c", "copy_snippet", "Copy Code"),
    ]

    def compose(self) -> ComposeResult:
        yield Header()
        with Horizontal():
            with Vertical(id="left-pane"):
                yield ListView(id="snippet-list")
            with Vertical(id="right-pane"):
                yield Markdown("Select a snippet from the list to view its contents.", id="snippet-view")
        yield Footer()

    def on_mount(self) -> None:
        self.conn = init_db()
        self.cursor = self.conn.cursor()
        self.action_refresh()

    def action_refresh(self) -> None:
        """Refresh the list of snippets."""
        list_view = self.query_one("#snippet-list", ListView)
        list_view.clear()
        
        self.cursor.execute("SELECT id, title FROM snippets ORDER BY created_at DESC")
        rows = self.cursor.fetchall()
        
        for row in rows:
            list_view.append(ListItem(Label(f"{row[0]} - {row[1]}"), id=f"item_{row[0]}"))
            
        if rows:
            list_view.index = 0

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
