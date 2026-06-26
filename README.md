# Competitive Programming Knowledge Base (CPKB)

![CPKB TUI Gallery](/intro-image.png)

CPKB is a local, terminal-first knowledge base designed to store, search, and track usages of competitive programming snippets, algorithms, and tricks. It uses SQLite for storage, keeping your snippets incredibly fast and perfectly organised.

## Version 2.0 Features

- **Store snippets**: Add code snippets with metadata like title, use case, and tags.
- **Search snippets**: Full-text search across titles, descriptions, tags, and code.
- **Track usage**: Record every time you use a snippet in a problem, linking to the file and optionally taking notes.
- **Textual TUI**: A beautiful terminal UI for browsing and copying your snippets.
- **FZF Integration**: Fuzzy find snippets directly in the terminal.
- **Snippet Insertion**: Instantly copy code to the clipboard or append to files.
- **Spaced Repetition (SM-2)**: Revise your knowledge base using the `revise` command with a true SM-2 spaced-repetition algorithm that tracks ease, interval, and repetitions per snippet.
- **Password-based Encryption**: Encrypt your database at rest with a password (PBKDF2-HMAC-SHA256 key derivation). No keys stored on disk.
- **XDG Base Directory compliant**: Stores your data safely in `~/.local/share/cpkb`.

## Installation

CPKB V2 is a proper Python package. For normal usage, install it via pip:

```bash
git clone https://github.com/Aaravshah2907/cpkb.git
cd cpkb
pip install -e .
```

### Local Development & Testing

If you wish to contribute to the codebase or run the unit tests locally to verify the CLI and TUI functionality, set up a virtual environment and run `pytest`:

```bash
# 1. Create and activate a virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows use: venv\Scripts\activate

# 2. Install CPKB and test dependencies
pip install -e .
pip install pytest pytest-asyncio pytest-mock cryptography

# 3. Run the test suite
pytest tests/
```

## Platform‑specific dependencies

| Platform | Clipboard utility   | Fuzzy finder                               |
|----------|---------------------|--------------------------------------------|
| macOS    | `pbcopy` (built‑in) | `brew install fzf`                         |
| Linux    | `xclip` or `xsel`   | `sudo apt-get install fzf`                 |
| Windows  | `clip` (built‑in)   | `scoop install fzf` or `choco install fzf` |

## Usage

Here are the commands available in Version 2.0:

### Core V1 Commands

- `cpkb add`: Add a new snippet interactively.
- `cpkb list`: List all snippets.
- `cpkb show <id>`: Show details, code, and usages of a specific snippet.
- `cpkb search <query>`: Search for snippets matching multiple words.
- `cpkb use <id> <file>`: Record the usage of a snippet in a specific file.
- `cpkb usages <id>`: List all recorded usages for a snippet.
- `cpkb stats`: Show basic database statistics.
- `cpkb random`: Show a random snippet for review or practice.
- `cpkb edit <id>`: Edit a snippet's metadata and code in your default `$EDITOR`.
- `cpkb edit-usage <id>`: Edit a past usage record.
- `cpkb delete <id>`: Delete a snippet permanently.
- `cpkb recent`: Show the 10 most recently added snippets.
- `cpkb export`: Export your entire knowledge base to a single Markdown file.
- `cpkb export-db [--encrypted]`: Export your SQLite database, optionally encrypted with a password.
- `cpkb import <file-or-url>`: Append snippets from a CPKB `db`, `json`, `md`, or `html` export.
- `cpkb import --defaults`: Import bundled C++ STL competitive-programming cheatsheets with special `cp_` IDs.
- `cpkb import --list-defaults`: Preview the bundled cheatsheets before importing.
- `cpkb backup`: Manually trigger a backup of the SQLite database.

### New V2 Commands

- `cpkb tui`: Launch the interactive Textual TUI (press `c` to copy a snippet).
- `cpkb fzf`: Interactively fuzzy search snippets using `fzf`.
- `cpkb copy <id> [-f <file>]`: Instantly copy the snippet's code to your system clipboard (uses `pbcopy` on macOS, `xclip`/`xsel` on Linux, `clip` on Windows) or append to a file.
- `cpkb revise`: Start a spaced-repetition session — the SM-2 algorithm selects the most overdue snippet, shows the title, then reveals the code. Rate your recall (0–5) to schedule the next review.
- `cpkb srs-stats`: Show spaced-repetition statistics (total reviewed, due now, avg ease factor).
- `cpkb encrypt-db`: Encrypt the database with a password (PBKDF2 + Fernet). No key files are stored.
- `cpkb decrypt-db`: Decrypt the database by entering your password.

## Directory Structure

Data is kept completely separate from the code repo. The application automatically creates the required directories on first run:

```txt
~/.local/share/cpkb/
├── snippets.db
├── attachments/
├── backups/
├── exports/
├── imports/
└── logs/
```

## Requirements

- Python 3.11+
- `textual`
- `cryptography` (for `encrypt-db` / `decrypt-db`)
- `fzf` (Optional, for `cpkb fzf`)
- macOS (uses `pbcopy` for clipboard interactions)

## License

Personal Knowledge Base (MIT License).
