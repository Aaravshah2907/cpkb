from textual.app import App, ComposeResult
from textual.widgets import Input, ListView, ListItem, Label
from textual import events

class TestApp(App):
    def compose(self) -> ComposeResult:
        yield Input(id="search")
        yield ListView(ListItem(Label("Test1")), ListItem(Label("Test2")), id="list")
    
    async def on_mount(self) -> None:
        self.bind("/", "focus_search")
        self.query_one("#list").focus()

    def action_focus_search(self) -> None:
        self.query_one("#search").focus()

if __name__ == "__main__":
    app = TestApp()
    # We can't easily simulate keypresses without async testing, so let's just write an async test script.
