# Competitive Programming Knowledge Base (CPKB)

![CPKB TUI Gallery](https://raw.githubusercontent.com/Aaravshah2907/cpkb/main/intro-image.png)

CPKB is a local, terminal-first knowledge base designed to store, search, and track usages of competitive programming snippets, algorithms, and tricks. It uses SQLite for storage, keeping your snippets incredibly fast and perfectly organised.

## Version 2.0 Features

- **Store snippets**: Add code snippets with metadata like title, use case, and tags.
- **Search snippets**: Full-text search across titles, descriptions, tags, and code.
- **Track usage**: Record every time you use a snippet in a problem, linking to the file and optionally taking notes.
- **Textual TUI**: A beautiful terminal UI for browsing and copying your snippets.
- **FZF Integration**: Fuzzy find snippets directly in the terminal.
- **Snippet Insertion**: Instantly copy code to the clipboard or append to files.
- **Spaced Repetition (SM-2)**: Revise your knowledge base using the `revise` command with a true SM-2 spaced-repetition algorithm that tracks ease, interval, and repetitions per snippet.
- **Optional Password-based Encryption**: Encrypt your database at rest with a password (PBKDF2-HMAC-SHA256 key derivation). No keys stored on disk. Requires the `cpkb[encrypt]` extra and `encryption.enabled` in config.
- **XDG Base Directory compliant**: Stores your data safely in `~/.local/share/cpkb`.

## Installation

The recommended install method is `pipx`:

```bash
pipx install cpkb
cpkb setup
```

You can also install it with `pip`:

```bash
python -m pip install cpkb
cpkb setup
```

The pip and pipx packages install the Python dependencies declared by CPKB, including Textual for `cpkb tui`.

You can safely run `cpkb setup` again later to revisit configuration; it preserves your snippet database.
Use `cpkb setup --reset-config` to start the prompts from factory defaults without deleting snippets.

### Homebrew

macOS and Linux users can install CPKB from the Homebrew tap:

```bash
brew tap Aaravshah2907/cpkb
brew trust --formula Aaravshah2907/cpkb/cpkb
brew install cpkb
cpkb setup
```

The Homebrew package runs CPKB with your global `python3` instead of a private Homebrew virtualenv, so Python packages you install globally are visible to `cpkb`. If `cpkb tui` reports that Textual is missing, install it into that same Python with the command printed by CPKB.

### Optional Encryption

Encryption support is opt-in because it depends on `cryptography`, which can make source-based package manager installs much heavier.

For encryption support, install with the `encrypt` extra:

```bash
pipx install "cpkb[encrypt]"
cpkb setup --enable-encryption
```

Then enable encryption in your CPKB config:

```bash
cpkb config
```

Open the printed `config.json` path and set:

```json
{
  "encryption": {
    "enabled": true
  }
}
```

If encryption is disabled or the extra is not installed, encrypted commands print the install/config steps instead of failing with a Python traceback.

### From Source

```bash
git clone https://github.com/Aaravshah2907/cpkb.git
cd cpkb
python -m pip install -e .
```

### Local Development & Testing

If you wish to contribute to the codebase or run the unit tests locally to verify the CLI and TUI functionality, set up a virtual environment and run `pytest`:

```bash
# 1. Create and activate a virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows use: venv\Scripts\activate

# 2. Install CPKB and test dependencies
python -m pip install -e ".[dev]"

# 3. Run the test suite
pytest
```

## Platform‑specific dependencies

| Platform | Clipboard utility   | Fuzzy finder                               |
|----------|---------------------|--------------------------------------------|
| macOS    | `pbcopy` (built‑in) | `brew install fzf`                         |
| Linux    | `xclip` or `xsel`   | `sudo apt-get install fzf`                 |
| Windows  | `clip` (built‑in)   | `scoop install fzf` or `choco install fzf` |

## Optional Integrations

CPKB includes an optional SketchyBar integration for macOS. It adds a menu bar item that opens a snippet search dialog and copies the selected snippet.

See [integrations/sketchybar](integrations/sketchybar) for the plugin script and `sketchybarrc` snippet.

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
- `cpkb setup`: Set up or revisit app directories, config, optional encryption settings, and bundled defaults without deleting snippets.
- `cpkb setup --reset-config`: Recreate config from factory defaults while preserving your snippet database.

### New V2 Commands

- `cpkb tui`: Launch the interactive Textual TUI (press `c` to copy a snippet).
- `cpkb fzf`: Interactively fuzzy search snippets using `fzf`.
- `cpkb copy <id> [-f <file>]`: Instantly copy the snippet's code to your system clipboard (uses `pbcopy` on macOS, `xclip`/`xsel` on Linux, `clip` on Windows) or append to a file.
- `cpkb revise`: Start a spaced-repetition session — the SM-2 algorithm selects the most overdue snippet, shows the title, then reveals the code. Rate your recall (0–5) to schedule the next review.
- `cpkb srs-stats`: Show spaced-repetition statistics (total reviewed, due now, avg ease factor).
- `cpkb encrypt-db`: Encrypt the database with a password (PBKDF2 + Fernet). Requires optional encryption support.
- `cpkb decrypt-db`: Decrypt the database by entering your password. Requires optional encryption support.

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
- `cryptography` via `cpkb[encrypt]` (optional, for `encrypt-db` / `decrypt-db`)
- `fzf` (Optional, for `cpkb fzf`)
- A platform clipboard helper: `pbcopy` on macOS, `xclip` or `xsel` on Linux, `clip` on Windows

## License

MIT License.
