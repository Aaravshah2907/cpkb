# CPKB (Competitive Programming Knowledge Base) - Comprehensive Codebase Knowledge Base

## 1. Overview & Architecture
**CPKB** is a terminal-first algorithm and snippet management system with a rich Textual-based TUI, scripting-friendly CLI, SQLite repository, spaced-repetition revision engine (SM-2 algorithm), multi-format import/export, and Neovim / VSCode / SketchyBar integrations.

### Architecture Diagram
```mermaid
graph TD
    CLI[cpkb CLI (src/cpkb/cli.py)] --> DB[(SQLite Database / src/cpkb/db.py)]
    TUI[Textual TUI (src/cpkb/tui.py)] --> DB
    CLI --> CONFIG[Config Manager (src/cpkb/config.py)]
    TUI --> CONFIG
    CLI --> DEFAULT[Default Snippets (src/cpkb/default_snippets.py)]
    NVIM[Neovim Telescope Extension] -->|cpkb query / cpkb show| CLI
    VSCODE[VSCode Extension] --> CLI
    SKETCHYBAR[Sketchybar Widget] --> CLI
```

---

## 2. Directory Structure & File Map

| Path | Purpose |
|------|---------|
| `src/cpkb/__init__.py` | Package version definition (`__version__ = "2.2.13"`) and module docstring. |
| `src/cpkb/config.py` | JSON configuration loading, merging defaults, path resolution, and type-safe helpers. |
| `src/cpkb/db.py` | SQLite schema management, automatic migrations, CRUD helpers, SRS (SM-2) engine, ID generators, backup/pruning. |
| `src/cpkb/cli.py` | CLI entry point (`argparse`), subcommand handlers, clipboard helpers, import/export parsers, sync engine. |
| `src/cpkb/tui.py` | Textual TUI application (`SnippetApp`), modal screens (Add, Edit, Delete, Tags, Use, Settings, Help, Filter, NewFormat, CustomTheme), reactive layout, Cosmere color resolver. |
| `src/cpkb/default_snippets.py` | Built-in C++ STL container and algorithm cheatsheets generator. |
| `src/cpkb/completions/` | Auto-completion definitions for bash, zsh, and fish shells. |
| `extras/plugins/nvim/` | Neovim Telescope plugin extension (`telescope/_extensions/cpkb.lua`). |
| `extras/plugins/vscode/` | VSCode snippet integration extension. |
| `extras/sketchybar/` | macOS SketchyBar integration scripts. |
| `tests/` | Pytest test suite covering CLI, TUI, database migrations, encryption, and vulture dead code analysis. |

---

## 3. Detailed File & Function Breakdown

### `src/cpkb/config.py`
- `DEFAULT_CONFIG: dict[str, Any]` : Default configuration schema containing display themes, snippet settings, keybindings, backups, and encryption flags.
- `_merge_defaults(defaults: dict, saved: dict) -> dict` : Recursively overlays user configuration over default dictionary to preserve newly added default options.
- `config_path(app_dir: Path) -> Path` : Returns path to `config.json` inside the specified application directory.
- `load_config(app_dir: Path) -> dict` : Loads and parses `config.json`, falling back safely to `DEFAULT_CONFIG` on error.
- `save_config(app_dir: Path, config: dict) -> Path` : Formats and persists configuration dict to `config.json`.
- `max_snippets(app_dir: Path) -> int` : Returns snippet limit (defaults to 9999).
- `max_backups(app_dir: Path) -> int` : Returns backup retention limit (defaults to 25).
- `encryption_enabled(app_dir: Path) -> bool` : Returns boolean indicating whether encryption features are active.

### `src/cpkb/db.py`
- **Schema & Connection Management**:
  - `get_conn() -> sqlite3.Connection`: Connects to `snippets.db` with `PRAGMA foreign_keys = ON`.
  - `_table_exists(cursor, table_name: str) -> bool`: Checks existence of a table.
  - `get_schema_version(cursor) -> int`: Reads version from `schema_meta` or infers legacy schema level (0, 1, 2).
  - `set_schema_version(cursor, version: int)`: Upserts version into `schema_meta`.
  - `_create_schema(cursor)`: Creates `snippets`, `usages`, `tags`, `reviews`, and `schema_meta` tables.
  - `_populate_tags_from_snippets(cursor)`: Migrates comma-separated string tags to normalized `tags` table rows.
  - `migrate_db(cursor, conn, db_existed: bool)`: Performs database backups and executes schema migrations.
  - `init_db() -> sqlite3.Connection`: Ensures required folders (`backups`, `exports`, `imports`, etc.) exist, saves initial config, and connects/migrates database.
  - `_now() -> str`: Generates current UTC ISO-8601 timestamp string.

- **ID Generation**:
  - `_normalize_id_pattern(pattern: str) -> str`: Normalizes bracket placeholders like `<ID_BEG_KEY>` and `<#######>`.
  - `_pattern_parts(format_config: dict) -> tuple[str, str, str]`: Computes prefix, digit placeholder string, and suffix.
  - `_id_format_config(format_name: str | None) -> tuple[str, int, str]`: Resolves configured prefix, width, and suffix.
  - `generate_id(cursor, format_name: str | None) -> str`: Generates the next sequential ID based on format rules.

- **Snippet CRUD & Search**:
  - `update_tags(cursor, snippet_id: str, tags_str: str)`: Normalizes and replaces tag rows for a snippet.
  - `add_snippet(cursor, conn, title, description, use_case, tags, code, id_format=None) -> str`: Inserts snippet, updates tags, commits, and returns generated ID.
  - `insert_snippet_with_id(cursor, conn, snippet_id, title, description, use_case, tags, code, created_at=None, updated_at=None) -> str`: Inserts snippet with caller-provided ID.
  - `import_snippets(cursor, conn, snippets: list[dict], preserve_ids=True, id_format=None) -> dict`: Appends batch snippets, avoiding overwrites and generating IDs when collisions occur.
  - `get_snippet(cursor, snippet_id: str) -> tuple | None`: Returns full snippet row.
  - `_validate_snippet_fields(fields) -> str`: Validates requested column names against whitelist `ALLOWED_SNIPPET_FIELDS`.
  - `get_snippet_fields(cursor, snippet_id: str, fields=...) -> tuple | None`: Fetches specific columns for a snippet.
  - `update_snippet(cursor, conn, snippet_id, title, description, use_case, tags, code)`: Updates snippet fields and refreshes tags.
  - `delete_snippet(cursor, conn, snippet_id)`: Backs up DB and deletes snippet (cascades to tags and usages).
  - `list_snippets(cursor) -> list[tuple]`: Returns `(id, title, tags)` for all snippets ordered by `created_at DESC`.
  - `recent_snippets(cursor, limit: int = 10) -> list[tuple]`: Returns top `limit` recent snippets.
  - `search_snippets(cursor, query: str) -> list[tuple]`: Multi-word AND search returning `(id, title, tags)`.
  - `search_snippets_full(cursor, query: str) -> list[tuple]`: Multi-word AND search returning `(id, title)`.
  - `get_random_snippet(cursor) -> tuple | None`: Selects 1 random snippet.
  - `get_all_snippet_ids(cursor) -> list[tuple]`: Returns all snippet IDs.

- **Spaced Repetition (SM-2 Algorithm)**:
  - `get_due_snippet(cursor) -> tuple | None`: Returns the most overdue snippet where `next_review <= now`.
  - `get_unreviewed_snippet(cursor) -> tuple | None`: Returns a snippet that has never been reviewed.
  - `upsert_review(cursor, conn, snippet_id: str, quality: int) -> dict`: Updates interval, repetitions, ease factor, and next review date.
  - `get_srs_stats(cursor) -> dict`: Calculates total, reviewed, unreviewed, due, and average ease factor.

- **Usages, Tags, Statistics & Backups**:
  - `add_usage(cursor, conn, snippet_id, file_path, problem_name, notes)`: Records snippet usage.
  - `get_usages(cursor, snippet_id) -> list[tuple]`: Retrieves all usage entries for a snippet.
  - `get_usage(cursor, usage_id) -> tuple | None`: Retrieves single usage entry.
  - `update_usage(cursor, conn, usage_id, file_path, problem_name, notes)`: Updates usage record.
  - `add_tag(cursor, conn, snippet_id, tag) -> str`: Adds a tag to snippet metadata.
  - `remove_tag(cursor, conn, snippet_id, tag) -> str | None`: Removes a tag from snippet metadata.
  - `get_stats(cursor) -> dict`: Returns snippet count, usage count, and distinct tag count.
  - `backup_db(prefix: str = "manual") -> str`: Copies database file to `backups/` with timestamp.
  - `prune_backups()`: Keeps backups within `max_backups` retention limit.

### `src/cpkb/cli.py`
- `_copy_to_clipboard(text: str)`: Cross-platform clipboard helper (`pbcopy`, `xclip`/`xsel`, `clip`).
- `cmd_backup(args)`: Creates a timestamped database backup.
- `cmd_config(args)`: Dumps active configuration in JSON format.
- `cmd_id_format_list(args)`: Lists all configured snippet ID formats and marks default.
- `cmd_id_format_add(args)`: Adds/updates an ID pattern/prefix in config.
- `cmd_id_format_default(args)`: Sets the default ID format.
- `cmd_setup(args)`: Interactive or non-interactive wizard configuring directory structure, language, display, completions, and encryption.
- `_require_encryption_available() -> bool`: Verifies cryptography library availability.
- `_derive_fernet_key(password, salt) -> bytes`: PBKDF2-HMAC-SHA256 key derivation.
- `cmd_encrypt_db(args)`: Password-encrypts `snippets.db` to `snippets.db.enc`.
- `cmd_decrypt_db(args)`: Password-decrypts `snippets.db.enc` back to `snippets.db`.
- `cmd_add(args)`: Interactive CLI prompt to input snippet title, description, use case, tags, and multi-line code.
- `cmd_edit(args)`: Launches `$EDITOR` with a temporary Markdown file containing metadata and code blocks.
- `cmd_delete(args)`: Prompts confirmation and deletes snippet.
- `cmd_list(args)`: Formatted terminal table output of all snippets.
- `cmd_recent(args)`: Displays recent snippets up to `-n/--limit`.
- `cmd_show(args)`: Prints full snippet metadata, code block, and usage history.
- `cmd_search(args)`: Multi-keyword AND search with table output.
- `cmd_query(args)`: Pipe/scripting-friendly search output in `id | title` format.
- `cmd_use(args)`: Prompts problem name and notes to record snippet usage.
- `cmd_usages(args)`: Displays table of recorded usages for a snippet.
- `cmd_edit_usage(args)`: Opens usage record in `$EDITOR`.
- `cmd_stats(args)`: Prints summary statistics.
- `cmd_random(args)`: Displays a randomly selected snippet.
- `cmd_revise(args)`: Interactive SM-2 spaced repetition flashcard session.
- `cmd_srs_stats(args)`: Prints SRS review statistics.
- `cmd_tag_add(args)` / `cmd_tag_remove(args)`: Modifies tags from CLI.
- `cmd_sync(args)`: Automates Git commit and push of application directory or rsync fallback.
- `cmd_export(args)` / `cmd_export_json(args)` / `cmd_export_html(args)` / `cmd_export_db(args)`: Multi-format exporters.
- `cmd_import(args)`: Multi-format importer supporting `.db`, `.json`, `.md`, `.html`, and encrypted `.db.enc`.
- `cmd_tui(args)`: Checks Textual installation and launches TUI.
- `cmd_fzf(args)`: Interactive fuzzy search using `fzf`.
- `cmd_copy(args)`: Copies snippet code to system clipboard or appends to a file.
- `cmd_install_completions(args)`: Installs shell completions for bash, zsh, or fish.
- `main()`: Argparse setup, command routing, and background update checker.

### `src/cpkb/tui.py`
- **Widgets & Helpers**:
  - `PathSuggester`: Auto-suggests file paths for usage inputs.
  - `CosmereColorSuggester`: Auto-suggests Cosmere palette color names.
  - `resolve_color(color_name: str) -> str`: Resolves named colors and Cosmere script variables.
  - `_sanitize_css_id(raw: str) -> str`: Sanitizes snippet IDs for Textual CSS compatibility.
- **Modal Screens**:
  - `AddSnippetModal`: Dialog for snippet ID format, title, description, use case, tags, and code (`TextArea`).
  - `EditSnippetModal`: Dialog for modifying existing snippet.
  - `ConfirmDeleteModal`: Error-themed confirmation modal.
  - `EditTagsModal`: Tag adder/remover modal.
  - `UseSnippetModal`: Modal to log snippet usage against a file path.
  - `NewFormatModal`: Dialog to create custom ID pattern or prefix/width.
  - `CustomThemeModal`: Color picker / theme definition dialog.
  - `SettingsModal`: Customizer for theme, accent, layout orientation, border style, and format color mappings.
  - `HelpModal`: Displays keybindings table.
  - `FilterTagModal`: Interactive tag filter picker.
- **SnippetApp**:
  - Main Textual application with split-pane layout (horizontal/vertical reactive styling).
  - Handles keybindings (`ctrl+q`, `ctrl+r`, `ctrl+c`, `/`, `ctrl+a`, `ctrl+e`, `ctrl+d`, `ctrl+t`, `ctrl+comma`, `j`/`k`, `[`, `]`, `?`, `f`, `tab`).
  - Real-time search filtering, dynamic Markdown rendering of snippet details, and format color tagging.

### `src/cpkb/default_snippets.py`
- `CHEATSHEETS`: Comprehensive C++ STL container and algorithm cheatsheet definitions (vector, pair, iterator, algorithm, string, stack, queue, priority_queue, deque, set, multiset, unordered_set, map, multimap, unordered_map, bitset, utility).
- `default_snippets(app_dir) -> list[dict]`: Generates formatted snippet dicts with comment blocks and markdown tables.

---

## 4. Summary of CLI & TUI Options

### Complete CLI Command Reference
- `cpkb add [--id-format FORMAT]`
- `cpkb list`
- `cpkb show <id>`
- `cpkb search <query>`
- `cpkb query <query> [--limit N]`
- `cpkb use <id> <file>`
- `cpkb usages <id>`
- `cpkb edit-usage <id>`
- `cpkb edit <id>`
- `cpkb delete <id>`
- `cpkb tag-add <id> <tag>`
- `cpkb tag-remove <id> <tag>`
- `cpkb recent [-n LIMIT]`
- `cpkb export` / `cpkb export-json` / `cpkb export-html` / `cpkb export-db [--encrypted]`
- `cpkb import [source] [--format {db,json,md,html}] [--encrypted] [--defaults] [--list-defaults] [--regenerate-ids] [--id-format FORMAT]`
- `cpkb backup`
- `cpkb config`
- `cpkb id-format {list,add,default}`
- `cpkb setup [-y] [--reset-config] [--load-defaults] [--enable-encryption] [--install-completions]`
- `cpkb encrypt-db` / `cpkb decrypt-db`
- `cpkb sync`
- `cpkb install-completions`
- `cpkb tui`
- `cpkb fzf`
- `cpkb copy <id> [-f FILE]`
- `cpkb revise`
- `cpkb srs-stats`

### Complete TUI Keybindings
- `ctrl+q` : Quit TUI
- `ctrl+r` : Refresh Snippet List
- `ctrl+c` : Copy Code of Selected Snippet to System Clipboard
- `/` : Focus Search Input Box
- `ctrl+a` : Open Add Snippet Modal
- `ctrl+e` : Open Edit Snippet Modal
- `ctrl+u` : Record Snippet Usage
- `ctrl+d` : Delete Snippet
- `ctrl+t` : Edit Tags Modal
- `ctrl+comma` : Settings Modal (Themes, Accents, Layout, Borders, Format Colors)
- `j` / `k` : Scroll Snippet Detail Pane Down / Up
- `pagedown` / `pageup` : Page Snippet Detail Pane Down / Up
- `[` / `]` : Shrink / Grow Left Pane Width
- `?` : Show Keybindings Help Modal
- `f` : Filter Snippets by Tag Modal
- `tab` : Switch Focus Between Search / List / Panes
