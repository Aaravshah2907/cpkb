# Codebase Knowledge Index & Map: CPKB

## 1. Architecture Overview
- **Type**: Competitive Programming Snippet Knowledge Base & TUI
- **Language**: Python 3.9+ (Core CLI & TUI) + Lua (Neovim plugin)
- **Database**: SQLite3 (`~/.local/share/cpkb/snippets.db`) with automatic schema versioning
- **Core Dependencies**: `textual` (TUI), standard library (`sqlite3`, `argparse`, `urllib`, `shutil`), optional `cryptography`

## 2. Directory & Module Map
- `/Users/aaravshah2975/cpkb/src/cpkb/`
  - `__init__.py`: Package entry and versioning.
  - `config.py`: Configuration persistence, default values, and theme attributes.
  - `db.py`: SQLite schema definitions, version migrations, snippet CRUD, tags, usages, and SM-2 spaced repetition engine.
  - `cli.py`: Argparse commands, clipboard integrations, import/export formats, and Git sync.
  - `tui.py`: Textual application, interactive modals (Add, Edit, Delete, Tags, Settings, Custom Theme, Help, Tag Filter).
  - `default_snippets.py`: Pre-bundled C++ STL cheatsheets.
- `/Users/aaravshah2975/cpkb/extras/plugins/nvim/`
  - `lua/telescope/_extensions/cpkb.lua`: Neovim Telescope picker for searching and inserting snippets.
- `/Users/aaravshah2975/.config/nvim/lua/plugins/cpkb.lua`
  - User Neovim plugin configuration mapping `<leader>cs` to `<cmd>Telescope cpkb<cr>`.

## 3. Schema & Data Model
- **Schema Version**: 2 (Migrating to 3 with language support)
- **Tables**:
  - `snippets`: `(id, title, description, use_case, tags, code, language, created_at, updated_at)`
  - `usages`: `(id, snippet_id, file_path, problem_name, notes, created_at)`
  - `tags`: `(id, snippet_id, tag)`
  - `reviews`: `(snippet_id, ease_factor, interval, repetitions, next_review, last_reviewed)`
  - `schema_meta`: `(key, value, updated_at)`
