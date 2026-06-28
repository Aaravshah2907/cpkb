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

prompt_yes_no() {
  prompt="$1"
  default="$2"
  answer="$(prompt_default "$prompt" "$default")"
  case "$(printf "%s" "$answer" | tr '[:upper:]' '[:lower:]')" in
    y|yes|true|1) return 0 ;;
    *) return 1 ;;
  esac
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

run_after_confirm() {
  prompt="$1"
  command_text="$2"
  default="${3:-n}"
  if prompt_yes_no "$prompt" "$default"; then
    printf "Running: %s\n" "$command_text"
    sh -c "$command_text"
  else
    printf "Skipped: %s\n" "$command_text"
  fi
}

find_python() {
  if command_exists python3; then
    printf "python3"
  elif command_exists python; then
    printf "python"
  else
    printf ""
  fi
}

install_linux_tools() {
  missing_clipboard="false"
  if ! command_exists xclip && ! command_exists xsel && ! command_exists wl-copy; then
    missing_clipboard="true"
  fi

  missing_fzf="false"
  if ! command_exists fzf; then
    missing_fzf="true"
  fi

  if [ "$missing_clipboard" = "false" ] && [ "$missing_fzf" = "false" ]; then
    printf "Clipboard helper and fzf are already available.\n"
    return
  fi

  if command_exists apt-get; then
    packages=""
    [ "$missing_clipboard" = "true" ] && packages="$packages xclip"
    [ "$missing_fzf" = "true" ] && packages="$packages fzf"
    run_after_confirm "Install missing Linux tools with apt-get? Packages:$packages" "sudo apt-get update && sudo apt-get install -y$packages" "n"
  elif command_exists dnf; then
    packages=""
    [ "$missing_clipboard" = "true" ] && packages="$packages xclip"
    [ "$missing_fzf" = "true" ] && packages="$packages fzf"
    run_after_confirm "Install missing Linux tools with dnf? Packages:$packages" "sudo dnf install -y$packages" "n"
  elif command_exists pacman; then
    packages=""
    [ "$missing_clipboard" = "true" ] && packages="$packages xclip"
    [ "$missing_fzf" = "true" ] && packages="$packages fzf"
    run_after_confirm "Install missing Linux tools with pacman? Packages:$packages" "sudo pacman -S --needed$packages" "n"
  else
    printf "Missing tools detected, but no supported package manager was found.\n"
    printf "Install one clipboard helper (xclip, xsel, or wl-clipboard) and optionally fzf.\n"
  fi
}

install_macos_tools() {
  if command_exists pbcopy; then
    printf "Clipboard helper pbcopy is already available.\n"
  fi
  if command_exists fzf; then
    printf "fzf is already available.\n"
  elif command_exists brew; then
    run_after_confirm "Install fzf with Homebrew?" "brew install fzf" "n"
  else
    printf "Homebrew was not found. Optional fuzzy finder install: brew install fzf\n"
  fi
}

install_windows_tools() {
  if command_exists clip; then
    printf "Clipboard helper clip is already available.\n"
  fi
  if command_exists fzf; then
    printf "fzf is already available.\n"
  elif command_exists scoop; then
    run_after_confirm "Install fzf with scoop?" "scoop install fzf" "n"
  elif command_exists choco; then
    run_after_confirm "Install fzf with Chocolatey?" "choco install fzf -y" "n"
  else
    printf "Optional fuzzy finder install: scoop install fzf, or choco install fzf\n"
  fi
}

APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/cpkb"
CONFIG_PATH="$APP_DIR/config.json"
PYTHON_BIN="$(find_python)"

if [ -z "$PYTHON_BIN" ]; then
  printf "Error: Python 3.11+ is required, but no python3/python executable was found.\n" >&2
  exit 1
fi

"$PYTHON_BIN" - <<'PY'
import sys
if sys.version_info < (3, 11):
    raise SystemExit("Error: Python 3.11+ is required.")
PY

mkdir -p "$APP_DIR/backups" "$APP_DIR/exports" "$APP_DIR/imports" "$APP_DIR/logs" "$APP_DIR/attachments"

DEFAULT_LANGUAGE="$(prompt_default "Default programming language" "cpp")"
MAX_SNIPPETS="$(prompt_default "Maximum number of snippets" "9999")"
MAX_BACKUPS="$(prompt_default "Maximum backups to keep" "25")"
THEME="$(prompt_default "TUI theme" "textual-dark")"
ACCENT_COLOR="$(prompt_default "Display accent color" "cyan")"
LOAD_CPP="$(prompt_default "Load bundled C++ cheatsheet on setup? (true/false)" "false")"
ENABLE_ENCRYPTION="$(prompt_default "Enable encryption commands? Requires optional cryptography dependency. (true/false)" "false")"

case "$(uname -s)" in
  Darwin)
    CLIPBOARD_NOTE="Clipboard: pbcopy is built in. Optional fuzzy finder: brew install fzf"
    install_macos_tools
    ;;
  Linux)
    CLIPBOARD_NOTE="Clipboard: install xclip or xsel. Optional fuzzy finder: sudo apt-get install fzf"
    install_linux_tools
    ;;
  MINGW*|MSYS*|CYGWIN*)
    CLIPBOARD_NOTE="Clipboard: clip is built in. Optional fuzzy finder: scoop install fzf or choco install fzf"
    install_windows_tools
    ;;
  *)
    CLIPBOARD_NOTE="Clipboard support depends on your OS. Optional fuzzy finder: install fzf"
    ;;
esac

"$PYTHON_BIN" - "$CONFIG_PATH" "$DEFAULT_LANGUAGE" "$MAX_SNIPPETS" "$MAX_BACKUPS" "$THEME" "$ACCENT_COLOR" "$LOAD_CPP" "$ENABLE_ENCRYPTION" <<'PY'
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
    "app_version": "2.1.2",
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
    "encryption": {
        "enabled": sys.argv[8].strip().lower() in {"1", "true", "yes", "y"},
    },
}

path.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
print(path)
PY

if prompt_yes_no "Install/update this checkout as the active cpkb command with '$PYTHON_BIN -m pip install -e .'?" "y"; then
  case "$(printf "%s" "$ENABLE_ENCRYPTION" | tr '[:upper:]' '[:lower:]')" in
    y|yes|true|1) "$PYTHON_BIN" -m pip install -e ".[encrypt]" ;;
    *) "$PYTHON_BIN" -m pip install -e . ;;
  esac
fi

if prompt_yes_no "Install optional test/development Python packages (pytest, pytest-asyncio, pytest-mock)?" "n"; then
  "$PYTHON_BIN" -m pip install pytest pytest-asyncio pytest-mock
fi

if command_exists cpkb; then
  printf "Active cpkb command: %s\n" "$(command -v cpkb)"
  cpkb config >/dev/null 2>&1 || printf "Warning: cpkb command exists but did not run cleanly yet.\n"
else
  printf "Warning: cpkb command is not on PATH. Try: %s -m pip install -e .\n" "$PYTHON_BIN"
fi

case "$(printf "%s" "$LOAD_CPP" | tr '[:upper:]' '[:lower:]')" in
  y|yes|true|1)
    if command_exists cpkb; then
      if prompt_yes_no "Import bundled C++ STL cheatsheets now?" "y"; then
        cpkb import --defaults
      fi
    else
      printf "Skipped bundled cheatsheet import because cpkb is not on PATH yet.\n"
    fi
    ;;
esac

printf "\nCPKB setup complete.\n"
printf "Config written to: %s\n" "$CONFIG_PATH"
printf "%s\n" "$CLIPBOARD_NOTE"
printf "Use '%s -m pip install -e .' from this repository if you need to refresh the cpkb command later.\n" "$PYTHON_BIN"
