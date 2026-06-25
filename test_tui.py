from textual.app import App
from cpkb.tui import SnippetApp
app = SnippetApp()
import asyncio
async def run_test():
    async with app.run_test() as pilot:
        await pilot.press("/")
        await pilot.press("a")
        await pilot.press("b")
        await pilot.press("c")
        await pilot.press("d")
        print("Test passed!")
asyncio.run(run_test())
