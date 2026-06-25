#!/usr/bin/env python3
"""
CPKB - Competitive Programming Knowledge Base
A terminal-first utility for storing and retrieving algorithm snippets.
"""

import argparse
import sqlite3
import sys
import os
import tempfile
import subprocess
import platform

def _copy_to_clipboard(text: str) -> None:
    """Copy *text* to the system clipboard in a cross‑platform way.

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
import shutil
from pathlib import Path
from datetime import datetime
import random as rnd

# XDG Base Directory specification
XDG_DATA_HOME = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share"))
APP_DIR = XDG_DATA_HOME / "cpkb"
DB_PATH = APP_DIR / "snippets.db"

def backup_db(prefix: str = "manual") -> str:
    if not DB_PATH.exists():
        return ""
    backups_dir = APP_DIR / "backups"
    backups_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y-%m-%d_%H%M%S")
    backup_path = backups_dir / f"snippets_{prefix}_{timestamp}.db"
    shutil.copy2(DB_PATH, backup_path)
    return str(backup_path)

def get_conn() -> sqlite3.Connection:
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA foreign_keys = ON")
    return conn

def init_db() -> sqlite3.Connection:
    """Initialize the directory structure and database schema."""
    APP_DIR.mkdir(parents=True, exist_ok=True)
    (APP_DIR / "backups").mkdir(exist_ok=True)
    (APP_DIR / "exports").mkdir(exist_ok=True)
    (APP_DIR / "imports").mkdir(exist_ok=True)
    (APP_DIR / "logs").mkdir(exist_ok=True)
    (APP_DIR / "attachments").mkdir(exist_ok=True)

    db_exists = DB_PATH.exists()
    conn = get_conn()
    cursor = conn.cursor()
    
    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='tags'")
    tags_exists = cursor.fetchone() is not None

    if db_exists and not tags_exists:
        print("Migrating database... Creating backup first.")
        backup_db("pre_migration")

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS snippets (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            use_case TEXT,
            tags TEXT,
            code TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS usages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snippet_id TEXT NOT NULL,
            file_path TEXT,
            problem_name TEXT,
            notes TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snippet_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            FOREIGN KEY(snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
        )
    ''')
    
    if db_exists and not tags_exists:
        cursor.execute("SELECT id, tags FROM snippets")
        for snip_id, tags_str in cursor.fetchall():
            if tags_str:
                for tag in [t.strip().lower() for t in tags_str.split(',') if t.strip()]:
                    cursor.execute("INSERT INTO tags (snippet_id, tag) VALUES (?, ?)", (snip_id, tag))
        print("Migration complete: populated tags table.")

    conn.commit()
    return conn

def generate_id(cursor: sqlite3.Cursor) -> str:
    """Generate the next snippet ID (e.g., CP0001)."""
    cursor.execute("SELECT id FROM snippets ORDER BY id DESC LIMIT 1")
    row = cursor.fetchone()
    if not row:
        return "CP0001"
    
    last_id = row[0]
    if last_id.startswith("CP") and last_id[2:].isdigit():
        num = int(last_id[2:])
        return f"CP{num + 1:04d}"
    
    cursor.execute("SELECT COUNT(*) FROM snippets")
    return f"CP{cursor.fetchone()[0] + 1:04d}"

def update_tags(cursor: sqlite3.Cursor, snippet_id: str, tags_str: str) -> None:
    cursor.execute("DELETE FROM tags WHERE snippet_id = ?", (snippet_id,))
    if tags_str:
        for tag in [t.strip().lower() for t in tags_str.split(',') if t.strip()]:
            cursor.execute("INSERT INTO tags (snippet_id, tag) VALUES (?, ?)", (snippet_id, tag))

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

    snippet_id = generate_id(cursor)
    now = datetime.utcnow().isoformat()

    cursor.execute('''
        INSERT INTO snippets (id, title, description, use_case, tags, code, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ''', (snippet_id, title, description, use_case, tags, code, now, now))
    
    update_tags(cursor, snippet_id, tags)
    conn.commit()

    print(f"\nSnippet added successfully! ID: {snippet_id}")

def cmd_edit(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    
    cursor.execute('SELECT title, description, use_case, tags, code FROM snippets WHERE id = ?', (args.id,))
    row = cursor.fetchone()
    
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
        
    now = datetime.utcnow().isoformat()
    cursor.execute('''
        UPDATE snippets
        SET title = ?, description = ?, use_case = ?, tags = ?, code = ?, updated_at = ?
        WHERE id = ?
    ''', (new_title, new_desc, new_use_case, new_tags, new_code, now, args.id))
    
    update_tags(cursor, args.id, new_tags)
    conn.commit()
    print(f"Snippet {args.id} updated successfully!")

def cmd_delete(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    
    cursor.execute('SELECT id, title FROM snippets WHERE id = ?', (args.id,))
    row = cursor.fetchone()
    if not row:
        print(f"Error: Snippet {args.id} not found.", file=sys.stderr)
        return
        
    confirm = input(f"Are you sure you want to delete '{row[1]}' ({args.id})? [y/N]: ").strip().lower()
    if confirm in ['y', 'yes']:
        backup_db("pre_delete")
        cursor.execute("DELETE FROM snippets WHERE id = ?", (args.id,))
        conn.commit()
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
    cursor.execute('SELECT id, title, tags FROM snippets ORDER BY created_at DESC')
    _print_list(cursor.fetchall())

def cmd_recent(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute('SELECT id, title, tags FROM snippets ORDER BY created_at DESC LIMIT ?', (args.limit,))
    print(f"Showing {args.limit} most recent snippets:")
    _print_list(cursor.fetchall())

def cmd_show(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute('SELECT * FROM snippets WHERE id = ?', (args.id,))
    row = cursor.fetchone()
    
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
    
    # Also display usages
    cursor.execute('''
        SELECT id, file_path, problem_name, notes, created_at 
        FROM usages 
        WHERE snippet_id = ? 
        ORDER BY created_at DESC
    ''', (args.id,))
    usages = cursor.fetchall()
    
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
    query_parts = args.query.lower().split()
    
    # Better search: find snippets that match ALL query parts
    conditions = []
    params = []
    for part in query_parts:
        like_str = f"%{part}%"
        conditions.append("(LOWER(title) LIKE ? OR LOWER(description) LIKE ? OR LOWER(tags) LIKE ? OR LOWER(code) LIKE ?)")
        params.extend([like_str, like_str, like_str, like_str])
        
    query_sql = "SELECT id, title, tags FROM snippets WHERE " + " AND ".join(conditions) + " ORDER BY created_at DESC"
    cursor.execute(query_sql, params)
    _print_list(cursor.fetchall())

def cmd_use(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    
    cursor.execute('SELECT id FROM snippets WHERE id = ?', (args.id,))
    if not cursor.fetchone():
        print(f"Error: Snippet {args.id} not found.", file=sys.stderr)
        return
        
    try:
        problem_name = input("Problem name (optional): ").strip()
        notes = input("Notes (optional): ").strip()
    except EOFError:
        print("\nAborted.")
        sys.exit(1)
        
    now = datetime.utcnow().isoformat()
    
    cursor.execute('''
        INSERT INTO usages (snippet_id, file_path, problem_name, notes, created_at)
        VALUES (?, ?, ?, ?, ?)
    ''', (args.id, args.file, problem_name, notes, now))
    conn.commit()
    print(f"Recorded usage of snippet {args.id} in {args.file}")

def cmd_usages(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute('''
        SELECT id, file_path, problem_name, notes, created_at 
        FROM usages 
        WHERE snippet_id = ? 
        ORDER BY created_at DESC
    ''', (args.id,))
    rows = cursor.fetchall()
    
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
    
    cursor.execute('SELECT file_path, problem_name, notes FROM usages WHERE id = ?', (args.id,))
    row = cursor.fetchone()
    
    if not row:
        print(f"Error: Usage with ID {args.id} not found.", file=sys.stderr)
        return
        
    file_path, problem_name, notes = row
    
    # Create a temporary file for editing
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
        
    cursor.execute('''
        UPDATE usages
        SET file_path = ?, problem_name = ?, notes = ?
        WHERE id = ?
    ''', (new_file_path, new_prob, new_notes, args.id))
    
    conn.commit()
    print(f"Usage {args.id} updated successfully!")

def cmd_stats(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    
    cursor.execute('SELECT COUNT(*) FROM snippets')
    snippet_count = cursor.fetchone()[0]
    
    cursor.execute('SELECT COUNT(*) FROM usages')
    usage_count = cursor.fetchone()[0]
    
    cursor.execute('SELECT COUNT(DISTINCT tag) FROM tags')
    tag_count = cursor.fetchone()[0]
    
    print("CPKB Statistics")
    print("-" * 20)
    print(f"Total Snippets: {snippet_count}")
    print(f"Total Usages:   {usage_count}")
    print(f"Unique Tags:    {tag_count}")

def cmd_random(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute('SELECT id FROM snippets')
    rows = cursor.fetchall()
    
    if not rows:
        print("No snippets found.")
        return
        
    random_id = rnd.choice(rows)[0]
    
    class DummyArgs:
        id = random_id
    cmd_show(DummyArgs())

def cmd_export(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute('SELECT id, title, description, use_case, tags, code FROM snippets ORDER BY created_at ASC')
    rows = cursor.fetchall()
    
    if not rows:
        print("No snippets to export.")
        return
        
    export_dir = APP_DIR / "exports"
    export_dir.mkdir(exist_ok=True)
    
    timestamp = datetime.now().strftime("%Y-%m-%d_%H%M%S")
    export_path = export_dir / f"snippets_{timestamp}.md"
    
    with open(export_path, 'w') as f:
        f.write("# CPKB Export\n\n")
        f.write(f"Generated at: {datetime.now().isoformat()}\n\n")
        
        for row in rows:
            f.write(f"## {row[0]}: {row[1]}\n\n")
            if row[2]: f.write(f"**Description:** {row[2]}\n\n")
            if row[3]: f.write(f"**Use Case:** {row[3]}\n\n")
            if row[4]: f.write(f"**Tags:** {row[4]}\n\n")
            f.write("```\n")
            f.write(row[5])
            f.write("\n```\n\n")
            f.write("---\n\n")
            
    print(f"Exported {len(rows)} snippets to {export_path}")

def cmd_backup(args: argparse.Namespace) -> None:
    path = backup_db("manual")
    if path:
        print(f"Database backed up successfully to: {path}")
    else:
        print("No database found to backup.")

def cmd_tui(args: argparse.Namespace) -> None:
    try:
        from .tui import run_tui
        run_tui()
    except ImportError:
        print("Error: Textual is not installed. Run 'pip install textual' first.", file=sys.stderr)

def cmd_fzf(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute("SELECT id, title, tags FROM snippets ORDER BY created_at DESC")
    rows = cursor.fetchall()
    
    if not rows:
        print("No snippets found.")
        return
        
    fzf_input = ""
    for r in rows:
        tags = r[2] if r[2] else ""
        fzf_input += f"{r[0]} | {r[1]} | {tags}\n"
        
    try:
        process = subprocess.Popen(['fzf', '--reverse', '--prompt', 'CPKB> '], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        stdout, _ = process.communicate(input=fzf_input)
        if stdout:
            selected_id = stdout.split('|')[0].strip()
            class DummyArgs: id = selected_id
            cmd_show(DummyArgs())
    except FileNotFoundError:
        print("Error: 'fzf' is not installed. Please install it (e.g., 'brew install fzf').", file=sys.stderr)

def cmd_copy(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute("SELECT code FROM snippets WHERE id = ?", (args.id,))
    row = cursor.fetchone()
    
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

def cmd_revise(args: argparse.Namespace) -> None:
    conn = init_db()
    cursor = conn.cursor()
    cursor.execute("SELECT id, title, description, code FROM snippets ORDER BY RANDOM() LIMIT 1")
    row = cursor.fetchone()
    
    if not row:
        print("No snippets found.")
        return
        
    print(f"Do you remember this snippet? [ID: {row[0]}]")
    print(f"Title:       {row[1]}")
    if row[2]:
        print(f"Description: {row[2]}")
        
    try:
        input("\nPress Enter to reveal the code...")
    except EOFError:
        print()
        sys.exit(0)
        
    print("\n--- Code ---\n")
    print(row[3])
    print("\n------------")

def main() -> None:
    parser = argparse.ArgumentParser(description="Competitive Programming Knowledge Base")
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
    
    parser_recent = subparsers.add_parser("recent", help="Show recent snippets")
    parser_recent.add_argument("-n", "--limit", type=int, default=10, help="Number of snippets to show")
    parser_recent.set_defaults(func=cmd_recent)
    
    parser_export = subparsers.add_parser("export", help="Export all snippets to a markdown file")
    parser_export.set_defaults(func=cmd_export)
    
    parser_backup = subparsers.add_parser("backup", help="Create a manual backup of the database")
    parser_backup.set_defaults(func=cmd_backup)

    # V2 Commands
    parser_tui = subparsers.add_parser("tui", help="Launch the Textual TUI")
    parser_tui.set_defaults(func=cmd_tui)
    
    parser_fzf = subparsers.add_parser("fzf", help="Search snippets using fzf")
    parser_fzf.set_defaults(func=cmd_fzf)
    
    parser_copy = subparsers.add_parser("copy", help="Copy a snippet to the clipboard or file")
    parser_copy.add_argument("id", help="Snippet ID")
    parser_copy.add_argument("-f", "--file", help="File to append to (optional)")
    parser_copy.set_defaults(func=cmd_copy)
    
    parser_revise = subparsers.add_parser("revise", help="Spaced repetition / random revision mode")
    parser_revise.set_defaults(func=cmd_revise)

    args = parser.parse_args()
    args.func(args)

if __name__ == "__main__":
    main()
