from textual.app import App, ComposeResult
from textual.widgets import Label

class TestApp(App):
    def compose(self) -> ComposeResult:
        yield Label("Test", id="test")

    async def on_mount(self) -> None:
        try:
            self.query_one("#test")
            print("Found in on_mount!")
        except Exception as e:
            print(f"Failed in on_mount: {e}")

if __name__ == "__main__":
    app = TestApp()
    app.run(headless=True)
