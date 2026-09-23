# CPKB Project — Status & Roadmap

## Completed in v3.0.x (Rust Native Edition)

- [x] **Full Rust Port (v3.0.0)**: Native Ratatui TUI and Clap CLI with SQLite backend. Zero Python runtime dependency.
- [x] **Database Deduplication**: Cleaned up 78 redundant/duplicate snippets from test re-imports with full data backups preserved.
- [x] **Custom Theme Color Editor**: In-TUI modal to customize all 11 hex color slots with live swatches (`●`/`✖`).
- [x] **Layout & Border Styles**: Horizontal/vertical pane toggle and 4 border styles (rounded, solid, double, thick) persisted in `config.json`.
- [x] **Natural Sorting**: Sort by snippet ID (`CP2 < CP10`), snippet name, or date.
- [x] **Quick Yank (`y`/`c`)**: Directly copy code from list in TUI with toast feedback.
- [x] **Language Filter (`L`)**: Cycle active languages in TUI with list title badge (`[Lang: cpp]`).
- [x] **Tag Selector Modal (`t`)**: Pop-up modal listing all distinct tags with counts to quickly filter snippets.
- [x] **JSON Output**: `--json` flag on `cpkb list`, `cpkb search`, `cpkb show` for editor/scripting integration.
- [x] **Shell Auto-installer**: `cpkb install-completions` dynamically detects shell (zsh/bash/fish) and installs completions.
- [x] **Optional Query**: `cpkb query` works with or without arguments for `fzf` piping.
- [x] **Automated Release Script**: `./scripts/release.sh [--dry-run] <version>` with Homebrew tap synchronization.
- [x] **Update All docs regarding python to rust change:** We need to update all of the appropriate docs as soon as possible.
- [x] **ID Format selector in Add/Edit View**: Add dropdown support for selecting/changing snippet ID format in TUI add and edit modal views.
