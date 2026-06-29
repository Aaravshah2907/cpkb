from textual.app import App, ComposeResult
from textual.containers import Vertical, VerticalScroll, Horizontal
from textual.widgets import Label, Button
from textual.screen import ModalScreen

class TestModal(ModalScreen):
    CSS = """
    .modal-dialog {
        padding: 1 2;
        padding-bottom: 4;
        width: 40;
        height: auto;
        max-height: 90%;
        border: thick red;
        background: blue;
        align: center middle;
    }
    .modal-body {
        height: auto;
        max-height: 1fr;
    }
    .modal-actions {
        dock: bottom;
        height: 3;
        width: 100%;
        background: green;
    }
    """
    def compose(self) -> ComposeResult:
        with Vertical(classes="modal-dialog"):
            yield Label("Title")
            with VerticalScroll(classes="modal-body"):
                for i in range(30):
                    yield Label(f"Line {i}")
            with Horizontal(classes="modal-actions"):
                yield Button("Save")

class TestApp(App):
    def on_mount(self):
        self.push_screen(TestModal())

if __name__ == "__main__":
    TestApp().run()
