#!/usr/bin/env sh
# setup.sh — CPKB v3.0.0 installer
#
# Installs the Rust-native cpkb binary as the primary entry point.
# The Python TUI (cpkb-py) is available as an optional legacy fallback.
#
set -eu

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()    { printf "${CYAN}▸${NC} %s\n" "$*"; }
success() { printf "${GREEN}✔${NC} %s\n" "$*"; }
die()     { printf "${RED}✖${NC} %s\n" "$*" >&2; exit 1; }

prompt_default() {
  printf "%s [%s]: " "$1" "$2" >&2
  read answer || answer=""
  if [ -z "$answer" ]; then printf "%s" "$2"; else printf "%s" "$answer"; fi
}

prompt_yes_no() {
  answer="$(prompt_default "$1" "$2")"
  case "$(printf "%s" "$answer" | tr '[:upper:]' '[:lower:]')" in
    y|yes|true|1) return 0 ;;
    *) return 1 ;;
  esac
}

command_exists() { command -v "$1" >/dev/null 2>&1; }

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/cpkb"
CONFIG_PATH="$APP_DIR/config.json"

# ── Pre-flight: Rust toolchain ───────────────────────────────────────────────

if ! command_exists cargo; then
  die "Rust toolchain not found. Install it from https://rustup.rs/ and re-run this script."
fi

# ── Create app directories ───────────────────────────────────────────────────

mkdir -p "$APP_DIR/backups" "$APP_DIR/exports" "$APP_DIR/imports" "$APP_DIR/logs" "$APP_DIR/attachments"
success "App directories ready at $APP_DIR"

# ── Build and install Rust binary ────────────────────────────────────────────

info "Building cpkb Rust binary (this may take a moment on first run)..."
(cd "$REPO_ROOT/rust" && cargo build --release)

RUST_BIN="$REPO_ROOT/rust/target/release/cpkb"
if [ ! -f "$RUST_BIN" ]; then
  die "Rust build succeeded but binary not found at $RUST_BIN"
fi

INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$INSTALL_DIR"
cp "$RUST_BIN" "$INSTALL_DIR/cpkb"
success "Installed cpkb binary to $INSTALL_DIR/cpkb"

# ── Interactive config setup ─────────────────────────────────────────────────

DEFAULT_LANGUAGE="$(prompt_default "Default programming language" "cpp")"
MAX_SNIPPETS="$(prompt_default "Maximum number of snippets" "9999")"
MAX_BACKUPS="$(prompt_default "Maximum backups to keep" "25")"
THEME="$(prompt_default "TUI theme (nord/dracula/gruvbox/catppuccin-mocha/cosmere/scadrial)" "nord")"
ACCENT_COLOR="$(prompt_default "Display accent color" "cyan")"
LAYOUT="$(prompt_default "Pane layout (horizontal/vertical)" "horizontal")"
BORDER_STYLE="$(prompt_default "Border style (round/solid/double/thick)" "round")"
SORT_BY="$(prompt_default "Default sort order (date/id/name)" "date")"
LOAD_CPP="$(prompt_default "Load bundled C++ cheatsheet on setup? (true/false)" "false")"

# Write config.json using the Rust binary's expected schema
cat > "$CONFIG_PATH" <<JSON
{
  "config_version": 1,
  "app_version": "3.0.1",
  "default_language": "$DEFAULT_LANGUAGE",
  "editor": {
    "command": "",
    "args": []
  },
  "display": {
    "theme": "$THEME",
    "accent_color": "$ACCENT_COLOR",
    "left_pane_width": 30,
    "layout": "$LAYOUT",
    "border_style": "$BORDER_STYLE",
    "sort_by": "$SORT_BY"
  },
  "snippets": {
    "max_number": $MAX_SNIPPETS,
    "id_formats": {
      "default": { "prefix": "CP", "digits": 4, "separator": "", "uppercase": true, "color": null },
      "ms":      { "prefix": "MS", "digits": 3, "separator": "-", "uppercase": true, "color": null },
      "latex":   { "prefix": "LATEX", "digits": 3, "separator": "-", "uppercase": true, "color": null }
    }
  },
  "backups": {
    "max_backups": $MAX_BACKUPS,
    "backup_on_exit": true
  },
  "imports": {
    "load_cpp_cheatsheet_on_setup": $(printf "%s" "$LOAD_CPP" | tr '[:upper:]' '[:lower:]')
  },
  "keybindings": {},
  "encryption": {
    "enabled": false
  }
}
JSON
success "Config written to $CONFIG_PATH"

# ── Optional: Python legacy TUI ──────────────────────────────────────────────

PYTHON_BIN=""
if command_exists python3; then PYTHON_BIN="python3"
elif command_exists python; then PYTHON_BIN="python"; fi

if [ -n "$PYTHON_BIN" ]; then
  if prompt_yes_no "Install Python legacy TUI as 'cpkb-py' (requires textual)?" "n"; then
    "$PYTHON_BIN" -m pip install -e . --quiet
    # Rename the pip-installed script so it doesn't shadow the Rust binary
    PY_SCRIPT="$INSTALL_DIR/cpkb-py"
    if command_exists cpkb && [ "$(command -v cpkb)" != "$INSTALL_DIR/cpkb" ]; then
      cp "$(command -v cpkb)" "$PY_SCRIPT" 2>/dev/null || true
    fi
    success "Python TUI available as 'cpkb-py'"
  fi
fi

# ── Optional: Import bundled C++ cheatsheet ──────────────────────────────────

case "$(printf "%s" "$LOAD_CPP" | tr '[:upper:]' '[:lower:]')" in
  y|yes|true|1)
    if command_exists cpkb; then
      if prompt_yes_no "Import bundled C++ STL cheatsheets now?" "y"; then
        cpkb import --defaults
      fi
    fi
    ;;
esac

# ── Verify ───────────────────────────────────────────────────────────────────

if command_exists cpkb; then
  INSTALLED_VERSION="$(cpkb --version 2>/dev/null || echo 'unknown')"
  success "cpkb installed: $INSTALLED_VERSION"
else
  printf "${RED}Warning:${NC} cpkb not found on PATH. Ensure %s is in your PATH:\n" "$INSTALL_DIR"
  printf "  export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
fi

printf "\n${GREEN}${BOLD}CPKB v3.0.0 setup complete!${NC}\n"
printf "  Start the TUI:   cpkb tui\n"
printf "  Add a snippet:   cpkb add\n"
printf "  List snippets:   cpkb list\n"
printf "  Help:            cpkb --help\n\n"
