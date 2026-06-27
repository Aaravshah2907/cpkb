#!/usr/bin/env python3
"""
CPKB - Competitive Programming Knowledge Base
A terminal-first utility for storing and retrieving algorithm snippets.
"""

import argparse
import sys
import os
import tempfile
import subprocess
import platform
import random as rnd
import shutil
import sqlite3
import importlib.util
from pathlib import Path
from urllib.parse import urlparse
from urllib.request import urlopen

from .db import (
    init_db, backup_db, generate_id, update_tags,
    add_snippet, get_snippet, get_snippet_fields, update_snippet,
    import_snippets,
    delete_snippet, list_snippets, recent_snippets, search_snippets,
    search_snippets_full,
    add_usage, get_usages, get_usage, update_usage,
    add_tag, remove_tag, get_stats, get_random_snippet, get_all_snippet_ids,
    get_due_snippet, get_unreviewed_snippet, upsert_review, get_srs_stats,
    APP_DIR, DB_PATH, KEY_PATH,
)
from .config import DEFAULT_CONFIG, encryption_enabled, load_config, save_config
from .default_snippets import default_snippets
from . import __version__


def _copy_to_clipboard(text: str) -> None:
    """Copy *text* to the system clipboard in a cross-platform way.

    Uses ``pbcopy`` on macOS, ``xclip``/``xsel`` on Linux, and ``clip`` on Windows.
    Raises ``RuntimeError`` if no suitable tool is found.
    """
    system = platform.system().lower()
    if system == "darwin":
        subprocess.run(["pbcopy"], input=text.encode())
    elif system == "linux":
        for cmd in (["xclip", "-selection", "clipboard"], ["xsel", "--clipboard"]):
            try:
                subprocess.run(cmd, input=text.encode(), check=True)
                return
            except FileNotFoundError:
                continue
        raise RuntimeError("Neither xclip nor xsel is installed; cannot copy to clipboard.")
    elif system == "windows":
        subprocess.run(["clip"], input=text.encode())
    else:
        raise RuntimeError(f"Unsupported OS: {system}")


# ---------------------------------------------------------------------------
# Backup
# ---------------------------------------------------------------------------

def cmd_backup(args: argparse.Namespace) -> None:
    """Create a manual backup of the database."""
    init_db()
    path = backup_db("manual")
    if path:
        print(f"Database backed up successfully to: {path}")
    else:
        print("No database found to backup.", file=sys.stderr)


def cmd_config(args: argparse.Namespace) -> None:
    """Print the active CPKB configuration path and values."""
    init_db()
    import json

    path = save_config(APP_DIR, load_config(APP_DIR))
    print(f"Config: {path}")
    print(json.dumps(load_config(APP_DIR), indent=2))


def _prompt_default(prompt: str, default: str) -> str:
    answer = input(f"{prompt} [{default}]: ").strip()
    return answer or default


def _as_bool(value: str) -> bool:
    return value.strip().lower() in {"1", "true", "yes", "y"}


def _as_int(value: str, default: int, minimum: int = 0) -> int:
    try:
        return max(minimum, int(value))
    except ValueError:
        return default


def _print_setup_tool_notes() -> None:
    system = platform.system().lower()
    if system == "darwin":
        print("Clipboard: pbcopy is built in. Optional fuzzy finder: brew install fzf")
    elif system == "linux":
        print("Clipboard: install xclip or xsel. Optional fuzzy finder: install fzf")
    elif system == "windows":
        print("Clipboard: clip is built in. Optional fuzzy finder: scoop install fzf or choco install fzf")
    else:
        print("Clipboard support depends on your OS. Optional fuzzy finder: install fzf")


def cmd_setup(args: argparse.Namespace) -> None:
    """Initialize CPKB app directories and config from the installed CLI."""
    config = load_config(APP_DIR)

    if args.yes:
        default_language = config.get("default_language", DEFAULT_CONFIG["default_language"])
        max_snippets_value = str(config.get("snippets", {}).get("max_number", 9999))
        max_backups_value = str(config.get("backups", {}).get("max_backups", 25))
        theme = config.get("display", {}).get("theme", "textual-dark")
        accent_color = config.get("display", {}).get("accent_color", "cyan")
        load_cpp = args.load_defaults
        enable_encryption = bool(args.enable_encryption)
    else:
        default_language = _prompt_default(
            "Default programming language",
            str(config.get("default_language", DEFAULT_CONFIG["default_language"])),
        )
        max_snippets_value = _prompt_default(
            "Maximum number of snippets",
            str(config.get("snippets", {}).get("max_number", 9999)),
        )
        max_backups_value = _prompt_default(
            "Maximum backups to keep",
            str(config.get("backups", {}).get("max_backups", 25)),
        )
        theme = _prompt_default(
            "TUI theme",
            str(config.get("display", {}).get("theme", "textual-dark")),
        )
        accent_color = _prompt_default(
            "Display accent color",
            str(config.get("display", {}).get("accent_color", "cyan")),
        )
        load_cpp = _as_bool(_prompt_default("Load bundled C++ cheatsheet on setup? (true/false)", "false"))
        enable_encryption = _as_bool(
            _prompt_default("Enable encryption commands? Requires optional cryptography dependency. (true/false)", "false")
        )

    APP_DIR.mkdir(parents=True, exist_ok=True)
    for subdir in ("backups", "exports", "imports", "logs", "attachments"):
        (APP_DIR / subdir).mkdir(exist_ok=True)

    updated_config = {
        **DEFAULT_CONFIG,
        "default_language": default_language,
        "display": {
            "theme": theme,
            "accent_color": accent_color,
        },
        "snippets": {
            "max_number": _as_int(max_snippets_value, 9999, 1),
        },
        "backups": {
            "max_backups": _as_int(max_backups_value, 25, 0),
        },
        "imports": {
            "load_cpp_cheatsheet_on_setup": bool(load_cpp),
        },
        "encryption": {
            "enabled": bool(enable_encryption),
        },
    }
    config_path = save_config(APP_DIR, updated_config)
    conn = init_db()
    conn.close()

    if load_cpp:
        class ImportDefaultsArgs:
            list_defaults = False
            defaults = True
            source = None
            format = None
            encrypted = False
            regenerate_ids = False

        cmd_import(ImportDefaultsArgs())

    print("\nCPKB setup complete.")
    print(f"Config written to: {config_path}")
    print(f"Active Python: {sys.executable}")
    _print_setup_tool_notes()


# ---------------------------------------------------------------------------
# Encryption
# ---------------------------------------------------------------------------

def _require_encryption_available() -> bool:
    """Return whether encryption is enabled and its optional dependency exists."""
    if not encryption_enabled(APP_DIR):
        print(
            "Encryption is disabled in CPKB config. Enable it by setting "
            '"encryption.enabled" to true in config.json, then install the '
            'optional dependency with: python3 -m pip install "cpkb[encrypt]"',
            file=sys.stderr,
        )
        return False

    try:
        import cryptography  # noqa: F401
    except ImportError:
        print(
            'Encryption support requires the optional dependency. Install it with: '
            'python3 -m pip install "cpkb[encrypt]"',
            file=sys.stderr,
        )
        return False

    return True

def _derive_fernet_key(password: str, salt: bytes) -> bytes:
    """Derive a Fernet-compatible key from *password* and *salt* using PBKDF2."""
    import base64
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives import hashes

    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=32,
        salt=salt,
        iterations=600_000,
    )
    return base64.urlsafe_b64encode(kdf.derive(password.encode()))


def cmd_encrypt_db(args: argparse.Namespace) -> None:
    """Encrypt the SQLite database using a password.

    Derives a Fernet key from the user's password via PBKDF2-HMAC-SHA256.
    A random 16-byte salt is generated and prepended to the encrypted output.
    No key or password is stored on disk — only the salt.
    """
    if not _require_encryption_available():
        return

    import getpass
    from cryptography.fernet import Fernet

    init_db()

    if not DB_PATH.exists():
        print("No database to encrypt.", file=sys.stderr)
        return

    # Prompt for password with confirmation
    password = getpass.getpass("Enter encryption password: ")
    if not password:
        print("Error: Password cannot be empty.", file=sys.stderr)
        return
    confirm = getpass.getpass("Confirm password: ")
    if password != confirm:
        print("Error: Passwords do not match.", file=sys.stderr)
        return

    # Backup before encryption
    bk = backup_db("pre_encrypt")
    if bk:
        print(f"Backup created at {bk}")

    # Generate a random salt and derive key
    salt = os.urandom(16)
    key = _derive_fernet_key(password, salt)

    with open(DB_PATH, "rb") as fdb:
        data = fdb.read()

    f = Fernet(key)
    encrypted = f.encrypt(data)

    # Write salt (16 bytes) + encrypted data
    enc_path = DB_PATH.with_suffix(".enc")
    with open(enc_path, "wb") as fe:
        fe.write(salt + encrypted)
    print(f"Encrypted database written to {enc_path}")

    # Clean up legacy plaintext key file if it exists
    if KEY_PATH.exists():
        KEY_PATH.unlink()
        print("Removed legacy plaintext key file.")
    legacy_loc = APP_DIR / "key_location.txt"
    if legacy_loc.exists():
        legacy_loc.unlink()


def cmd_decrypt_db(args: argparse.Namespace) -> None:
    """Decrypt the previously encrypted SQLite database using a password."""
    if not _require_encryption_available():
        return

    import getpass
    from cryptography.fernet import Fernet

    init_db()

    enc_path = DB_PATH.with_suffix(".enc")
    if not enc_path.exists():
        print("No encrypted database found.", file=sys.stderr)
        return

    password = getpass.getpass("Enter decryption password: ")
    if not password:
        print("Error: Password cannot be empty.", file=sys.stderr)
        return

    # Backup before decryption
    bk = backup_db("pre_decrypt")
    if bk:
        print(f"Backup created at {bk}")

    with open(enc_path, "rb") as fe:
        raw = fe.read()

    # First 16 bytes are the salt
    salt = raw[:16]
    encrypted = raw[16:]

    key = _derive_fernet_key(password, salt)
    f = Fernet(key)
    try:
        data = f.decrypt(encrypted)
    except Exception:
        print("Decryption failed: wrong password or corrupted file.", file=sys.stderr)
        return

    with open(DB_PATH, "wb") as fdb:
        fdb.write(data)
    print(f"Decrypted database restored to {DB_PATH}")


# ---------------------------------------------------------------------------
# Snippet Commands
# ---------------------------------------------------------------------------

def cmd_add(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()

    print("Adding a new snippet...")
    try:
        title = input("Title: ").strip()
        description = input("Description: ").strip()
        use_case = input("Use case: ").strip()
        tags = input("Tags (comma separated): ").strip()

        print("Enter the code (Ctrl+D on an empty line to finish):")
        lines = sys.stdin.readlines()
        code = "".join(lines).strip()
    except EOFError:
        print("\nAborted.")
        sys.exit(1)

    if not title or not code:
        print("Error: Title and code are required.", file=sys.stderr)
        sys.exit(1)

    snippet_id = add_snippet(cursor, conn, title, description, use_case, tags, code)
    print(f"\nSnippet added successfully! ID: {snippet_id}")


def cmd_edit(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()

    row = get_snippet_fields(cursor, args.id)
    if not row:
        print(f"Error: Snippet {args.id} not found.", file=sys.stderr)
        return

    title, description, use_case, tags, code = row

    # Create a temporary file for editing
    with tempfile.NamedTemporaryFile(mode='w+', suffix='.md', delete=False) as tf:
        tf.write(f"Title: {title}\n")
        tf.write(f"Description: {description}\n")
        tf.write(f"Use case: {use_case}\n")
        tf.write(f"Tags: {tags}\n")
        tf.write("---\n")
        tf.write(code)
        temp_path = tf.name

    editor = os.environ.get('EDITOR', 'nano')
    subprocess.call([editor, temp_path])

    with open(temp_path, 'r') as tf:
        content = tf.read()
    os.remove(temp_path)

    parts = content.split('---', 1)
    if len(parts) != 2:
        print("Error: Invalid format after edit. Make sure '---' separator is intact.", file=sys.stderr)
        return

    metadata_lines = parts[0].strip().split('\n')
    new_code = parts[1].strip()

    new_title, new_desc, new_use_case, new_tags = "", "", "", ""
    for line in metadata_lines:
        if line.lower().startswith('title:'):
            new_title = line.split(':', 1)[1].strip()
        elif line.lower().startswith('description:'):
            new_desc = line.split(':', 1)[1].strip()
        elif line.lower().startswith('use case:'):
            new_use_case = line.split(':', 1)[1].strip()
        elif line.lower().startswith('tags:'):
            new_tags = line.split(':', 1)[1].strip()

    if not new_title or not new_code:
        print("Error: Title and code cannot be empty.", file=sys.stderr)
        return

    update_snippet(cursor, conn, args.id, new_title, new_desc, new_use_case, new_tags, new_code)
    print(f"Snippet {args.id} updated successfully!")


def cmd_delete(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()

    row = get_snippet_fields(cursor, args.id, "id, title")
    if not row:
        print(f"Error: Snippet {args.id} not found.", file=sys.stderr)
        return

    confirm = input(f"Are you sure you want to delete '{row[1]}' ({args.id})? [y/N]: ").strip().lower()
    if confirm in ['y', 'yes']:
        delete_snippet(cursor, conn, args.id)
        print(f"Snippet {args.id} deleted.")
    else:
        print("Aborted.")


def _print_list(rows: list) -> None:
    if not rows:
        print("No snippets found.")
        return
    print(f"{'ID':<10} {'Title':<40} {'Tags'}")
    print("-" * 70)
    for row in rows:
        title = row[1] if len(row[1]) <= 38 else row[1][:35] + "..."
        tags = row[2] if row[2] else ""
        print(f"{row[0]:<10} {title:<40} {tags}")


def cmd_list(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    _print_list(list_snippets(cursor))


def cmd_recent(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    print(f"Showing {args.limit} most recent snippets:")
    _print_list(recent_snippets(cursor, args.limit))


def cmd_show(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()

    row = get_snippet(cursor, args.id)
    if not row:
        print(f"Error: Snippet with ID {args.id} not found.", file=sys.stderr)
        return

    print(f"ID:          {row[0]}")
    print(f"Title:       {row[1]}")
    print(f"Description: {row[2]}")
    print(f"Use Case:    {row[3]}")
    print(f"Tags:        {row[4]}")
    print(f"Created At:  {row[6]}")
    print(f"Updated At:  {row[7]}")
    print("\n--- Code ---\n")
    print(row[5])
    print("\n------------")

    usages = get_usages(cursor, args.id)
    if usages:
        print(f"\nUsages ({len(usages)}):")
        for u in usages:
            date_str = u[4][:10]
            prob = f" - {u[2]}" if u[2] else ""
            notes = f" ({u[3]})" if u[3] else ""
            print(f"  • [ID: {u[0]}] {date_str}: {u[1]}{prob}{notes}")
    else:
        print("\nUsages (0): None recorded yet.")


def cmd_search(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    _print_list(search_snippets(cursor, args.query))

def cmd_query(args: argparse.Namespace) -> None:
    """Scripting-friendly query command that outputs 'id | title'."""
    conn = init_db()
    cursor = conn.cursor()
    
    rows = search_snippets_full(cursor, args.query)
    for row in rows[:args.limit]:
        print(f"{row[0]} | {row[1]}")


# ---------------------------------------------------------------------------
# Usage Commands
# ---------------------------------------------------------------------------

def cmd_use(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()

    row = get_snippet_fields(cursor, args.id, "id")
    if not row:
        print(f"Error: Snippet {args.id} not found.", file=sys.stderr)
        return

    try:
        problem_name = input("Problem name (optional): ").strip()
        notes = input("Notes (optional): ").strip()
    except EOFError:
        print("\nAborted.")
        sys.exit(1)

    add_usage(cursor, conn, args.id, args.file, problem_name, notes)
    print(f"Recorded usage of snippet {args.id} in {args.file}")


def cmd_usages(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    rows = get_usages(cursor, args.id)

    if not rows:
        print(f"No usages found for snippet {args.id}.")
        return

    print(f"Usages for {args.id}:")
    print(f"{'ID':<6} {'Date':<12} {'File':<25} {'Problem':<20}")
    print("-" * 70)
    for row in rows:
        date_str = row[4][:10]
        file_path = row[1] if len(row[1]) <= 23 else "..." + row[1][-20:]
        problem = row[2] if row[2] else ""
        problem = problem if len(problem) <= 18 else problem[:15] + "..."

        print(f"{row[0]:<6} {date_str:<12} {file_path:<25} {problem:<20}")
        if row[3]:
            print(f"  Notes: {row[3]}")


def cmd_edit_usage(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()

    row = get_usage(cursor, args.id)
    if not row:
        print(f"Error: Usage with ID {args.id} not found.", file=sys.stderr)
        return

    file_path, problem_name, notes = row

    with tempfile.NamedTemporaryFile(mode='w+', suffix='.md', delete=False) as tf:
        tf.write(f"File Path: {file_path}\n")
        tf.write(f"Problem Name: {problem_name or ''}\n")
        tf.write(f"Notes: {notes or ''}\n")
        temp_path = tf.name

    editor = os.environ.get('EDITOR', 'nano')
    subprocess.call([editor, temp_path])

    with open(temp_path, 'r') as tf:
        content = tf.read()
    os.remove(temp_path)

    lines = content.strip().split('\n')
    new_file_path, new_prob, new_notes = "", "", ""
    for line in lines:
        if line.lower().startswith('file path:'):
            new_file_path = line.split(':', 1)[1].strip()
        elif line.lower().startswith('problem name:'):
            new_prob = line.split(':', 1)[1].strip()
        elif line.lower().startswith('notes:'):
            new_notes = line.split(':', 1)[1].strip()

    if not new_file_path:
        print("Error: File path cannot be empty.", file=sys.stderr)
        return

    update_usage(cursor, conn, args.id, new_file_path, new_prob, new_notes)
    print(f"Usage {args.id} updated successfully!")


# ---------------------------------------------------------------------------
# Stats / Random / Revise
# ---------------------------------------------------------------------------

def cmd_stats(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    stats = get_stats(cursor)

    print("CPKB Statistics")
    print("-" * 20)
    print(f"Total Snippets: {stats['snippets']}")
    print(f"Total Usages:   {stats['usages']}")
    print(f"Unique Tags:    {stats['tags']}")


def cmd_random(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    ids = get_all_snippet_ids(cursor)

    if not ids:
        print("No snippets found.")
        return

    random_id = rnd.choice(ids)[0]

    class DummyArgs:
        id = random_id
    cmd_show(DummyArgs())


def cmd_revise(args: argparse.Namespace) -> None:
    """True spaced-repetition revision using the SM-2 algorithm.

    Prioritises overdue snippets, then falls back to never-reviewed
    snippets, then picks a random one.  After revealing the code the
    user rates their recall quality (0-5) and the schedule is updated.
    """
    conn = init_db()
    cursor = conn.cursor()

    # 1. Pick the most overdue snippet
    row = get_due_snippet(cursor)
    source = "due for review"

    # 2. If nothing is due, pick a never-reviewed snippet
    if not row:
        row = get_unreviewed_snippet(cursor)
        source = "never reviewed"

    # 3. Last resort: any random snippet
    if not row:
        row = get_random_snippet(cursor)
        source = "random"

    if not row:
        print("No snippets found. Add some with 'cpkb add'.")
        return

    snippet_id, title, description, code = row

    print(f"\n📖  Revise this snippet ({source})  [ID: {snippet_id}]")
    print(f"Title:       {title}")
    if description:
        print(f"Description: {description}")

    try:
        input("\nPress Enter to reveal the code...")
    except EOFError:
        print()
        sys.exit(0)

    print("\n--- Code ---\n")
    print(code)
    print("\n------------")

    # Quality rating
    print("\nRate your recall quality:")
    print("  0 — Complete blackout")
    print("  1 — Wrong, but recognised the answer")
    print("  2 — Wrong, but answer felt familiar")
    print("  3 — Correct, but with serious difficulty")
    print("  4 — Correct, with some hesitation")
    print("  5 — Perfect recall")

    try:
        raw = input("\nQuality [0-5]: ").strip()
    except EOFError:
        print("\nSkipped rating.")
        return

    if not raw.isdigit() or int(raw) not in range(6):
        print("Invalid rating — skipping schedule update.")
        return

    quality = int(raw)
    result = upsert_review(cursor, conn, snippet_id, quality)

    print(f"\n✅  Schedule updated for {snippet_id}:")
    print(f"   Next review in {result['interval']} day(s)")
    print(f"   Ease factor: {result['ease_factor']}")
    print(f"   Repetitions: {result['repetitions']}")


def cmd_srs_stats(args: argparse.Namespace) -> None:
    """Display spaced-repetition statistics."""
    conn = init_db()
    cursor = conn.cursor()
    stats = get_srs_stats(cursor)

    print("\n📊  Spaced-Repetition Statistics")
    print("-" * 35)
    print(f"Total snippets:   {stats['total_snippets']}")
    print(f"Reviewed:         {stats['reviewed']}")
    print(f"Never reviewed:   {stats['never_reviewed']}")
    print(f"Due now:          {stats['due_now']}")
    if stats['avg_ease_factor'] is not None:
        print(f"Avg ease factor:  {stats['avg_ease_factor']}")
    else:
        print("Avg ease factor:  N/A (no reviews yet)")
    print()


# ---------------------------------------------------------------------------
# Tag Commands
# ---------------------------------------------------------------------------

def cmd_tag_add(args: argparse.Namespace) -> None:
    """Add a tag to a snippet."""
    conn = init_db()
    cursor = conn.cursor()
    try:
        add_tag(cursor, conn, args.id, args.tag)
        print(f"Tag '{args.tag}' added to snippet {args.id}.")
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)


def cmd_tag_remove(args: argparse.Namespace) -> None:
    """Remove a tag from a snippet."""
    conn = init_db()
    cursor = conn.cursor()
    try:
        result = remove_tag(cursor, conn, args.id, args.tag)
        if result is None:
            print(f"Tag '{args.tag}' not present on snippet {args.id}.")
        else:
            print(f"Tag '{args.tag}' removed from snippet {args.id}.")
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)


# ---------------------------------------------------------------------------
# Sync (Git-based)
# ---------------------------------------------------------------------------

def cmd_sync(args: argparse.Namespace) -> None:
    """Sync the CPKB data directory to a Git remote.

    Initialises a Git repository inside ``~/.local/share/cpkb`` (if not
    already present), commits all changes, and pushes to ``origin``.

    Set the remote once with:
        git -C ~/.local/share/cpkb remote add origin <url>

    Falls back to rsync if ``CPKB_SYNC_TARGET`` is set (legacy behaviour).
    """
    init_db()

    # Legacy rsync fallback
    rsync_target = os.getenv("CPKB_SYNC_TARGET")

    git_dir = APP_DIR / ".git"

    if not git_dir.exists() and not rsync_target:
        # First-time setup: initialise a git repo
        try:
            subprocess.run(["git", "init"], cwd=str(APP_DIR), check=True,
                           capture_output=True, text=True)
            print(f"Initialized Git repository in {APP_DIR}")
            print("Add a remote with:  git -C ~/.local/share/cpkb remote add origin <url>")
            print("Then run 'cpkb sync' again to push.")
        except FileNotFoundError:
            print("Error: 'git' is not installed. Install Git first.", file=sys.stderr)
        return

    if git_dir.exists():
        # Git-based sync
        try:
            # Create a .gitignore if it doesn't exist
            gitignore = APP_DIR / ".gitignore"
            if not gitignore.exists():
                gitignore.write_text("backups/\nlogs/\n*.enc\nencryption.key\nkey_location.txt\n")

            # Stage all changes
            subprocess.run(["git", "add", "-A"], cwd=str(APP_DIR), check=True,
                           capture_output=True, text=True)

            # Check if there are changes to commit
            result = subprocess.run(["git", "status", "--porcelain"], cwd=str(APP_DIR),
                                    capture_output=True, text=True, check=True)
            if not result.stdout.strip():
                print("Nothing to sync — database is up to date.")
                return

            # Commit
            from datetime import datetime
            msg = f"cpkb sync {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
            subprocess.run(["git", "commit", "-m", msg], cwd=str(APP_DIR), check=True,
                           capture_output=True, text=True)
            print(f"Committed changes: {msg}")

            # Check if a remote is configured
            result = subprocess.run(["git", "remote"], cwd=str(APP_DIR),
                                    capture_output=True, text=True, check=True)
            if not result.stdout.strip():
                print("No remote configured. Add one with:")
                print(f"  git -C {APP_DIR} remote add origin <url>")
                print("Changes have been committed locally.")
                return

            # Push
            subprocess.run(["git", "push", "-u", "origin", "HEAD"], cwd=str(APP_DIR),
                           check=True, capture_output=True, text=True)
            print("Synced to remote successfully!")

        except subprocess.CalledProcessError as e:
            stderr = e.stderr if e.stderr else str(e)
            print(f"Git sync failed: {stderr}", file=sys.stderr)

    elif rsync_target:
        # Legacy rsync sync
        if not DB_PATH.exists():
            print("No database to sync.", file=sys.stderr)
            return
        cmd = ["rsync", "-avz", str(DB_PATH), rsync_target]
        try:
            subprocess.run(cmd, check=True)
            print(f"Database synced to {rsync_target}")
        except subprocess.CalledProcessError as e:
            print(f"Sync failed: {e}", file=sys.stderr)

    # Always create a local backup after sync
    path = backup_db("post_sync")
    if path:
        print(f"Local backup created at: {path}")


# ---------------------------------------------------------------------------
# Export Commands
# ---------------------------------------------------------------------------

def cmd_export(args: argparse.Namespace) -> None:
    """Export all snippets to a markdown file."""
    from datetime import datetime
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM snippets ORDER BY created_at")
    rows = cursor.fetchall()
    if not rows:
        print("No snippets to export.")
        return
    export_dir = APP_DIR / "exports"
    export_dir.mkdir(parents=True, exist_ok=True)
    out_path = export_dir / f'snippets_{datetime.utcnow().strftime("%Y%m%d_%H%M%S")}.md'
    with open(out_path, 'w') as f:
        for row in rows:
            f.write(f"## {row[1]} ({row[0]})\n")
            f.write(f"**Description:** {row[2] or ''}\n")
            f.write(f"**Use case:** {row[3] or ''}\n")
            f.write(f"**Tags:** {row[4] or ''}\n")
            f.write("\n```\n" + row[5] + "\n```\n\n")
    print(f"Exported {len(rows)} snippets to {out_path}")


def cmd_export_json(args: argparse.Namespace) -> None:
    """Export snippets to JSON file."""
    import json
    from datetime import datetime
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM snippets ORDER BY created_at")
    rows = cursor.fetchall()
    data = [
        {
            "id": r[0], "title": r[1], "description": r[2], "use_case": r[3],
            "tags": r[4], "code": r[5], "created_at": r[6], "updated_at": r[7],
        } for r in rows
    ]
    export_dir = APP_DIR / "exports"
    export_dir.mkdir(parents=True, exist_ok=True)
    out_path = export_dir / f'snippets_{datetime.utcnow().strftime("%Y%m%d_%H%M%S")}.json'
    with open(out_path, 'w') as f:
        json.dump(data, f, indent=2)
    print(f"Exported {len(data)} snippets to {out_path}")


def cmd_export_html(args: argparse.Namespace) -> None:
    """Export snippets to an HTML file."""
    from datetime import datetime
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM snippets ORDER BY created_at")
    rows = cursor.fetchall()
    export_dir = APP_DIR / "exports"
    export_dir.mkdir(parents=True, exist_ok=True)
    out_path = export_dir / f'snippets_{datetime.utcnow().strftime("%Y%m%d_%H%M%S")}.html'
    with open(out_path, 'w') as f:
        f.write("<html><head><meta charset='utf-8'><title>CPKB Export</title></head><body>")
        for row in rows:
            f.write(f"<section><h2>{row[1]} ({row[0]})</h2>")
            f.write(f"<p><strong>Description:</strong> {row[2] or ''}</p>")
            f.write(f"<p><strong>Use case:</strong> {row[3] or ''}</p>")
            f.write(f"<p><strong>Tags:</strong> {row[4] or ''}</p>")
            f.write(f"<pre>{row[5]}</pre></section><hr/>")
        f.write("</body></html>")
    print(f"Exported {len(rows)} snippets to {out_path}")


def cmd_export_db(args: argparse.Namespace) -> None:
    """Export the SQLite database, optionally encrypted with a password."""
    from datetime import datetime, timezone

    init_db()
    if not DB_PATH.exists():
        print("No database to export.", file=sys.stderr)
        return

    export_dir = APP_DIR / "exports"
    export_dir.mkdir(parents=True, exist_ok=True)

    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    if args.encrypted:
        if not _require_encryption_available():
            return

        import getpass
        from cryptography.fernet import Fernet

        password = getpass.getpass("Enter export encryption password: ")
        if not password:
            print("Error: Password cannot be empty.", file=sys.stderr)
            return
        confirm = getpass.getpass("Confirm password: ")
        if password != confirm:
            print("Error: Passwords do not match.", file=sys.stderr)
            return

        salt = os.urandom(16)
        key = _derive_fernet_key(password, salt)
        with open(DB_PATH, "rb") as fdb:
            data = fdb.read()

        out_path = export_dir / f"snippets_{timestamp}.db.enc"
        with open(out_path, "wb") as fe:
            fe.write(salt + Fernet(key).encrypt(data))
    else:
        out_path = export_dir / f"snippets_{timestamp}.db"
        shutil.copy2(DB_PATH, out_path)

    print(f"Exported database to {out_path}")


# ---------------------------------------------------------------------------
# Import Commands
# ---------------------------------------------------------------------------

def _snippet_dict_from_row(row: tuple) -> dict:
    return {
        "id": row[0],
        "title": row[1],
        "description": row[2] or "",
        "use_case": row[3] or "",
        "tags": row[4] or "",
        "code": row[5],
        "created_at": row[6],
        "updated_at": row[7],
    }


def _read_source(source: str) -> tuple[bytes, str]:
    parsed = urlparse(source)
    if parsed.scheme in {"http", "https"}:
        with urlopen(source, timeout=20) as response:
            return response.read(), Path(parsed.path).name
    path = Path(source)
    return path.read_bytes(), path.name


def _decrypt_export(raw: bytes, password: str) -> bytes:
    if not _require_encryption_available():
        raise RuntimeError(
            'Encryption support is disabled or unavailable. Enable encryption in '
            'config.json and install with: python3 -m pip install "cpkb[encrypt]"'
        )

    from cryptography.fernet import Fernet

    salt = raw[:16]
    encrypted = raw[16:]
    key = _derive_fernet_key(password, salt)
    return Fernet(key).decrypt(encrypted)


def _load_json_snippets(raw: bytes) -> list[dict]:
    import json

    data = json.loads(raw.decode("utf-8"))
    if not isinstance(data, list):
        raise ValueError("JSON import must contain a list of snippets.")
    return [item for item in data if isinstance(item, dict)]


def _load_db_snippets(raw: bytes) -> list[dict]:
    with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as tf:
        tf.write(raw)
        temp_path = tf.name

    try:
        conn = sqlite3.connect(temp_path)
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM snippets ORDER BY created_at")
        rows = cursor.fetchall()
        conn.close()
    finally:
        os.remove(temp_path)

    return [_snippet_dict_from_row(row) for row in rows]


def _load_markdown_snippets(raw: bytes) -> list[dict]:
    import re

    text = raw.decode("utf-8")
    pattern = re.compile(
        r"^## (?P<title>.+?) \((?P<id>[^)]+)\)\n"
        r"\*\*Description:\*\* (?P<description>.*?)\n"
        r"\*\*Use case:\*\* (?P<use_case>.*?)\n"
        r"\*\*Tags:\*\* (?P<tags>.*?)\n\n"
        r"```\n(?P<code>.*?)\n```\n?",
        re.MULTILINE | re.DOTALL,
    )
    return [match.groupdict() for match in pattern.finditer(text)]


def _load_html_snippets(raw: bytes) -> list[dict]:
    import html
    import re

    text = raw.decode("utf-8")
    section_pattern = re.compile(r"<section>(?P<section>.*?)</section>", re.DOTALL)
    snippets = []
    for section_match in section_pattern.finditer(text):
        section = section_match.group("section")
        title_match = re.search(r"<h2>(?P<title>.*?) \((?P<id>.*?)\)</h2>", section, re.DOTALL)
        desc_match = re.search(r"<p><strong>Description:</strong> (?P<value>.*?)</p>", section, re.DOTALL)
        use_match = re.search(r"<p><strong>Use case:</strong> (?P<value>.*?)</p>", section, re.DOTALL)
        tags_match = re.search(r"<p><strong>Tags:</strong> (?P<value>.*?)</p>", section, re.DOTALL)
        code_match = re.search(r"<pre>(?P<code>.*?)</pre>", section, re.DOTALL)
        if not title_match or not code_match:
            continue
        snippets.append(
            {
                "id": html.unescape(title_match.group("id").strip()),
                "title": html.unescape(title_match.group("title").strip()),
                "description": html.unescape(desc_match.group("value").strip()) if desc_match else "",
                "use_case": html.unescape(use_match.group("value").strip()) if use_match else "",
                "tags": html.unescape(tags_match.group("value").strip()) if tags_match else "",
                "code": html.unescape(code_match.group("code").strip()),
            }
        )
    return snippets


def _detect_import_format(source_name: str, explicit: str | None) -> str:
    if explicit:
        return explicit
    lowered = source_name.lower()
    if lowered.endswith(".db.enc"):
        return "db"
    if lowered.endswith(".db"):
        return "db"
    if lowered.endswith(".json"):
        return "json"
    if lowered.endswith(".md") or lowered.endswith(".markdown"):
        return "md"
    if lowered.endswith(".html") or lowered.endswith(".htm"):
        return "html"
    raise ValueError("Could not detect import format. Use --format db|json|md|html.")


def _load_import_snippets(source: str, fmt: str | None, encrypted: bool) -> list[dict]:
    import getpass

    raw, source_name = _read_source(source)
    if encrypted or source_name.lower().endswith(".enc"):
        password = getpass.getpass("Enter import decryption password: ")
        raw = _decrypt_export(raw, password)

    detected = _detect_import_format(source_name.removesuffix(".enc"), fmt)
    if detected == "db":
        return _load_db_snippets(raw)
    if detected == "json":
        return _load_json_snippets(raw)
    if detected == "md":
        return _load_markdown_snippets(raw)
    if detected == "html":
        return _load_html_snippets(raw)
    raise ValueError(f"Unsupported import format: {detected}")


def cmd_import(args: argparse.Namespace) -> None:
    """Append snippets from a CPKB export or bundled defaults."""
    if args.list_defaults:
        snippets = default_snippets(APP_DIR)
        for snippet in snippets:
            print(f"{snippet['id']} | {snippet['title']}")
        return

    conn = init_db()
    cursor = conn.cursor()

    if args.defaults:
        snippets = default_snippets(APP_DIR)
    else:
        if not args.source:
            print("Import failed: provide a source path/URL or use --defaults.", file=sys.stderr)
            return
        try:
            snippets = _load_import_snippets(args.source, args.format, args.encrypted)
        except Exception as exc:
            print(f"Import failed: {exc}", file=sys.stderr)
            return

    result = import_snippets(cursor, conn, snippets, preserve_ids=not args.regenerate_ids)
    print(f"Imported {result['imported']} snippet(s); skipped {result['skipped']}.")
    collisions = sum(1 for old, new in result["id_map"].items() if old != new)
    if collisions:
        print(f"Regenerated {collisions} colliding ID(s).")


# ---------------------------------------------------------------------------
# TUI / FZF / Copy
# ---------------------------------------------------------------------------

def cmd_tui(args: argparse.Namespace) -> None:
    if importlib.util.find_spec("textual") is None:
        print(
            "Error: Textual is not installed for the Python running cpkb.",
            file=sys.stderr,
        )
        print(
            f"Install it with: {sys.executable} -m pip install textual",
            file=sys.stderr,
        )
        if "Cellar" in sys.executable and "libexec" in sys.executable:
            print(
                "This looks like a Homebrew private virtualenv, so global pip installs "
                "will not be visible to this cpkb command.",
                file=sys.stderr,
            )
        return

    try:
        from .tui import run_tui
        run_tui()
    except ImportError as exc:
        print(f"Error: failed to import the Textual TUI: {exc}", file=sys.stderr)


def cmd_fzf(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    rows = list_snippets(cursor)

    if not rows:
        print("No snippets found.")
        return

    fzf_input = ""
    for r in rows:
        tags = r[2] if r[2] else ""
        fzf_input += f"{r[0]} | {r[1]} | {tags}\n"

    try:
        process = subprocess.Popen(
            ['fzf', '--reverse', '--prompt', 'CPKB> '],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True
        )
        stdout, _ = process.communicate(input=fzf_input)
        if stdout:
            selected_id = stdout.split('|')[0].strip()
            class DummyArgs: id = selected_id
            cmd_show(DummyArgs())
    except FileNotFoundError:
        if platform.system().lower() == "windows":
            print("Error: 'fzf' is not installed. Install it via 'scoop install fzf' or 'choco install fzf'.", file=sys.stderr)
        else:
            print("Error: 'fzf' is not installed. Please install it (e.g., 'brew install fzf').", file=sys.stderr)


def cmd_copy(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()

    row = get_snippet_fields(cursor, args.id, "code")
    if not row:
        print(f"Error: Snippet {args.id} not found.", file=sys.stderr)
        return

    if args.file:
        with open(args.file, "a") as f:
            f.write("\n" + row[0] + "\n")
        print(f"Snippet {args.id} appended to {args.file}")
    else:
        try:
            _copy_to_clipboard(row[0])
            print(f"Snippet {args.id} copied to clipboard!")
        except Exception as e:
            print(f"Clipboard copy failed ({e}); here is the code:\n")
            print(row[0])


# ---------------------------------------------------------------------------
# CLI Entry Point
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(prog="cpkb", description="Competitive Programming Knowledge Base")
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # V1 Commands
    parser_add = subparsers.add_parser("add", help="Add a new snippet")
    parser_add.set_defaults(func=cmd_add)

    parser_list = subparsers.add_parser("list", help="List all snippets")
    parser_list.set_defaults(func=cmd_list)

    parser_show = subparsers.add_parser("show", help="Show a specific snippet")
    parser_show.add_argument("id", help="Snippet ID (e.g., CP0001)")
    parser_show.set_defaults(func=cmd_show)

    parser_search = subparsers.add_parser("search", help="Search snippets")
    parser_search.add_argument("query", help="Search query (multiple words will be AND'ed)")
    parser_search.set_defaults(func=cmd_search)

    parser_query = subparsers.add_parser("query", help="Search snippets (scripting friendly)")
    parser_query.add_argument("query", help="Search query")
    parser_query.add_argument("--limit", type=int, default=5, help="Number of results to return")
    parser_query.set_defaults(func=cmd_query)

    parser_use = subparsers.add_parser("use", help="Record usage of a snippet")
    parser_use.add_argument("id", help="Snippet ID")
    parser_use.add_argument("file", help="File where the snippet is used")
    parser_use.set_defaults(func=cmd_use)

    parser_usages = subparsers.add_parser("usages", help="List usages of a snippet")
    parser_usages.add_argument("id", help="Snippet ID")
    parser_usages.set_defaults(func=cmd_usages)

    parser_stats = subparsers.add_parser("stats", help="Show knowledge base statistics")
    parser_stats.set_defaults(func=cmd_stats)

    parser_random = subparsers.add_parser("random", help="Show a random snippet")
    parser_random.set_defaults(func=cmd_random)

    # V1.1 & V1.2 Commands
    parser_edit = subparsers.add_parser("edit", help="Edit a snippet in your default $EDITOR")
    parser_edit.add_argument("id", help="Snippet ID")
    parser_edit.set_defaults(func=cmd_edit)

    parser_edit_usage = subparsers.add_parser("edit-usage", help="Edit a specific usage record")
    parser_edit_usage.add_argument("id", type=int, help="Usage ID (integer)")
    parser_edit_usage.set_defaults(func=cmd_edit_usage)

    parser_delete = subparsers.add_parser("delete", help="Delete a snippet")
    parser_delete.add_argument("id", help="Snippet ID")
    parser_delete.set_defaults(func=cmd_delete)

    parser_tag_add = subparsers.add_parser("tag-add", help="Add a tag to a snippet")
    parser_tag_add.add_argument("id", help="Snippet ID")
    parser_tag_add.add_argument("tag", help="Tag to add")
    parser_tag_add.set_defaults(func=cmd_tag_add)

    parser_tag_remove = subparsers.add_parser("tag-remove", help="Remove a tag from a snippet")
    parser_tag_remove.add_argument("id", help="Snippet ID")
    parser_tag_remove.add_argument("tag", help="Tag to remove")
    parser_tag_remove.set_defaults(func=cmd_tag_remove)

    parser_recent = subparsers.add_parser("recent", help="Show recent snippets")
    parser_recent.add_argument("-n", "--limit", type=int, default=10, help="Number of snippets to show")
    parser_recent.set_defaults(func=cmd_recent)

    parser_export = subparsers.add_parser("export", help="Export all snippets to a markdown file")
    parser_export.set_defaults(func=cmd_export)
    parser_export_json = subparsers.add_parser("export-json", help="Export snippets to JSON")
    parser_export_json.set_defaults(func=cmd_export_json)
    parser_export_html = subparsers.add_parser("export-html", help="Export snippets to HTML")
    parser_export_html.set_defaults(func=cmd_export_html)
    parser_export_db = subparsers.add_parser("export-db", help="Export the SQLite database")
    parser_export_db.add_argument("--encrypted", action="store_true", help="Encrypt exported DB with a password")
    parser_export_db.set_defaults(func=cmd_export_db)

    parser_import = subparsers.add_parser("import", help="Import snippets from a CPKB export")
    parser_import.add_argument("source", nargs="?", help="Path or URL to a CPKB db/json/md/html export")
    parser_import.add_argument("--format", choices=["db", "json", "md", "html"], help="Import format")
    parser_import.add_argument("--encrypted", action="store_true", help="Decrypt source before importing")
    parser_import.add_argument("--defaults", action="store_true", help="Import bundled C++ STL cheatsheets")
    parser_import.add_argument("--list-defaults", action="store_true", help="Preview bundled cheatsheets")
    parser_import.add_argument("--regenerate-ids", action="store_true", help="Generate new CP IDs instead of preserving source IDs")
    parser_import.set_defaults(func=cmd_import)

    parser_backup = subparsers.add_parser("backup", help="Create a manual backup of the database")
    parser_backup.set_defaults(func=cmd_backup)

    parser_config = subparsers.add_parser("config", help="Show active configuration")
    parser_config.set_defaults(func=cmd_config)

    parser_setup = subparsers.add_parser("setup", help="Set up CPKB directories, config, and optional defaults")
    parser_setup.add_argument("-y", "--yes", action="store_true", help="Accept current/default config values without prompts")
    parser_setup.add_argument("--load-defaults", action="store_true", help="Import bundled C++ STL cheatsheets")
    parser_setup.add_argument("--enable-encryption", action="store_true", help="Enable encryption commands in config")
    parser_setup.set_defaults(func=cmd_setup)

    parser_encrypt = subparsers.add_parser("encrypt-db", help="Encrypt the database with a password")
    parser_encrypt.set_defaults(func=cmd_encrypt_db)

    parser_decrypt = subparsers.add_parser("decrypt-db", help="Decrypt the database with a password")
    parser_decrypt.set_defaults(func=cmd_decrypt_db)

    parser_sync = subparsers.add_parser("sync", help="Sync database to Git remote (or rsync)")
    parser_sync.set_defaults(func=cmd_sync)

    # V2 Commands
    parser_tui = subparsers.add_parser("tui", help="Launch the Textual TUI")
    parser_tui.set_defaults(func=cmd_tui)

    parser_fzf = subparsers.add_parser("fzf", help="Search snippets using fzf")
    parser_fzf.set_defaults(func=cmd_fzf)

    parser_copy = subparsers.add_parser("copy", help="Copy a snippet to the clipboard or file")
    parser_copy.add_argument("id", help="Snippet ID")
    parser_copy.add_argument("-f", "--file", help="File to append to (optional)")
    parser_copy.set_defaults(func=cmd_copy)

    parser_revise = subparsers.add_parser("revise", help="Spaced-repetition revision (SM-2 algorithm)")
    parser_revise.set_defaults(func=cmd_revise)

    parser_srs_stats = subparsers.add_parser("srs-stats", help="Show spaced-repetition statistics")
    parser_srs_stats.set_defaults(func=cmd_srs_stats)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
