#!/usr/bin/env sh
set -eu

prompt_default() {
  prompt="$1"
  default="$2"
  printf "%s [%s]: " "$prompt" "$default" >&2
  read answer || answer=""
  if [ -z "$answer" ]; then
    printf "%s" "$default"
  else
    printf "%s" "$answer"
  fi
}

APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/cpkb"
CONFIG_PATH="$APP_DIR/config.json"

mkdir -p "$APP_DIR/backups" "$APP_DIR/exports" "$APP_DIR/imports" "$APP_DIR/logs" "$APP_DIR/attachments"

DEFAULT_LANGUAGE="$(prompt_default "Default programming language" "cpp")"
MAX_SNIPPETS="$(prompt_default "Maximum number of snippets" "9999")"
MAX_BACKUPS="$(prompt_default "Maximum backups to keep" "25")"
THEME="$(prompt_default "TUI theme" "textual-dark")"
ACCENT_COLOR="$(prompt_default "Display accent color" "cyan")"
LOAD_CPP="$(prompt_default "Load bundled C++ cheatsheet on setup? (true/false)" "false")"

case "$(uname -s)" in
  Darwin)
    CLIPBOARD_NOTE="Clipboard: pbcopy is built in. Optional fuzzy finder: brew install fzf"
    ;;
  Linux)
    CLIPBOARD_NOTE="Clipboard: install xclip or xsel. Optional fuzzy finder: sudo apt-get install fzf"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    CLIPBOARD_NOTE="Clipboard: clip is built in. Optional fuzzy finder: scoop install fzf or choco install fzf"
    ;;
  *)
    CLIPBOARD_NOTE="Clipboard support depends on your OS. Optional fuzzy finder: install fzf"
    ;;
esac

python3 - "$CONFIG_PATH" "$DEFAULT_LANGUAGE" "$MAX_SNIPPETS" "$MAX_BACKUPS" "$THEME" "$ACCENT_COLOR" "$LOAD_CPP" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])

def as_int(value, default, minimum=0):
    try:
        return max(minimum, int(value))
    except ValueError:
        return default

config = {
    "config_version": 1,
    "app_version": "2.0.1",
    "default_language": sys.argv[2] or "cpp",
    "display": {
        "theme": sys.argv[5] or "textual-dark",
        "accent_color": sys.argv[6] or "cyan",
    },
    "snippets": {
        "max_number": as_int(sys.argv[3], 9999, 1),
    },
    "backups": {
        "max_backups": as_int(sys.argv[4], 25),
    },
    "imports": {
        "load_cpp_cheatsheet_on_setup": sys.argv[7].strip().lower() in {"1", "true", "yes", "y"},
    },
}

path.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
print(path)
PY

printf "\nCPKB setup complete.\n"
printf "Config written to: %s\n" "$CONFIG_PATH"
printf "%s\n" "$CLIPBOARD_NOTE"
printf "Run 'pip install -e .' from this repository if the cpkb command is not available yet.\n"
