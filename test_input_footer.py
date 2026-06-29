from textual.app import App, ComposeResult
from textual.widgets import Input, Footer

class TestApp(App):
    BINDINGS = [("quit", "ctrl+q", "Quit"), ("focus_next", "tab", "Switch Panes")]
    def compose(self) -> ComposeResult:
        yield Input()
        yield Footer()

if __name__ == "__main__":
    app = TestApp()
    # just print bindings
    print(app._bindings.keys)
