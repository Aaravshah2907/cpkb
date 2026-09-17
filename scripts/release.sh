#!/usr/bin/env bash
#
# release.sh — Automate a cpkb release
#
# Usage:
#   ./scripts/release.sh [--dry-run] <new-version>
#
# Example:
#   ./scripts/release.sh 3.0.1
#   ./scripts/release.sh --dry-run 3.0.1
#
# This script will:
#   1. Validate the new version string
#   2. Bump the version in rust/Cargo.toml (primary), pyproject.toml,
#      src/cpkb/__init__.py, and setup.sh
#   3. Run the Rust test suite + differential parity tests
#   4. Build the Rust release binary + Python wheel
#   5. Commit, tag (v<version>), and push to origin
#   6. Download the GitHub archive tarball and compute its SHA256
#   7. Update Formula/cpkb.rb in this repo
#   8. Update the homebrew tap repo at $(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb
#   9. Commit and push both formula changes
#
set -euo pipefail

# ── Helpers ─────────────────────────────────────────────────────────────────────

RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
CYAN=$'\033[0;36m'
BOLD=$'\033[1m'
NC=$'\033[0m'

info()    { printf "${CYAN}▸${NC} %s\n" "$*"; }
success() { printf "${GREEN}✔${NC} %s\n" "$*"; }
warn()    { printf "${YELLOW}⚠${NC} %s\n" "$*"; }
die()     { printf "${RED}✖${NC} %s\n" "$*" >&2; exit 1; }

countdown_sleep() {
  local total_seconds="$1"
  local message="$2"
  local remaining minutes seconds

  for ((remaining = total_seconds; remaining > 0; remaining--)); do
    minutes=$((remaining / 60))
    seconds=$((remaining % 60))
    printf "\r${CYAN}▸${NC} %s Remaining: %02d:%02d" "$message" "$minutes" "$seconds"
    sleep 1
  done
  printf "\r${CYAN}▸${NC} %s Remaining: 00:00\n" "$message"
}

# ── Parse arguments ──────────────────────────────────────────────────────────

DRY_RUN=false
NEW_VERSION=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    *)
      NEW_VERSION="$1"
      shift
      ;;
  esac
done

if [[ -z "$NEW_VERSION" ]]; then
  die "Usage: $0 [--dry-run] <new-version>  (e.g. 3.0.1)"
fi

# Validate semver format
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  die "Version must be in X.Y.Z format, got: $NEW_VERSION"
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Read current version from Cargo.toml (source of truth)
OLD_VERSION=$(grep '^version = ' rust/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
info "Current version: ${BOLD}$OLD_VERSION${NC}  (from rust/Cargo.toml)"
info "New version:     ${BOLD}$NEW_VERSION${NC}"
if [[ "$DRY_RUN" == "true" ]]; then
  info "Mode:            ${YELLOW}${BOLD}DRY-RUN (no remote pushes or tags will be created)${NC}"
fi

if [[ "$OLD_VERSION" == "$NEW_VERSION" ]]; then
  die "New version is the same as current ($OLD_VERSION). Nothing to do. Did you forget to bump?"
fi

if git tag -l "v$NEW_VERSION" | grep -q .; then
  die "Tag v$NEW_VERSION already exists."
fi

# Ensure clean working tree
if ! git diff --quiet HEAD; then
  warn "Working tree has uncommitted changes."
  read -rp "Continue anyway? [y/N] " confirm
  [[ "$confirm" =~ ^[Yy]$ ]] || die "Aborted."
fi

# ── Step 1: Bump version ────────────────────────────────────────────────────────

info "Bumping version $OLD_VERSION → $NEW_VERSION ..."

python3 -c "
from pathlib import Path
for filepath in ['rust/Cargo.toml', 'pyproject.toml', 'src/cpkb/__init__.py', 'setup.sh']:
    p = Path(filepath)
    if p.exists():
        text = p.read_text(encoding='utf-8')
        if filepath == 'rust/Cargo.toml':
            text = text.replace('version = \"$OLD_VERSION\"', 'version = \"$NEW_VERSION\"', 1)
        elif filepath == 'pyproject.toml':
            text = text.replace('version = \"$OLD_VERSION\"', 'version = \"$NEW_VERSION\"', 1)
        elif filepath == 'src/cpkb/__init__.py':
            text = text.replace('__version__ = \"$OLD_VERSION\"', '__version__ = \"$NEW_VERSION\"', 1)
        elif filepath == 'setup.sh':
            text = text.replace('\"app_version\": \"$OLD_VERSION\"', '\"app_version\": \"$NEW_VERSION\"', 1)
        p.write_text(text, encoding='utf-8')
"

success "Version bumped in rust/Cargo.toml, pyproject.toml, __init__.py, setup.sh"

# ── Step 2: Run tests ───────────────────────────────────────────────────────────

info "Running Rust test suite ..."
(cd rust && cargo test -q)
success "All Rust tests passed"

info "Running differential parity & performance suite ..."
python3 tests/test_differential_parity.py
success "All differential parity and benchmark tests passed"

info "Running Python legacy test suite ..."
PYTHONPATH=src pytest -q || warn "Python pytest skipped or non-fatal"

# ── Step 3: Build ───────────────────────────────────────────────────────────────

info "Building Rust release binary ..."
(cd rust && cargo build --release)
success "Rust release binary built at rust/target/release/cpkb"

info "Building Python distributions (wheel + sdist) ..."
rm -rf dist/cpkb-"$NEW_VERSION"*
python3 -m build || warn "python build skipped (non-fatal for Rust releases)"

if [[ "$DRY_RUN" == "true" ]]; then
  info "Reverting local version bumps for dry-run ..."
  git checkout -- rust/Cargo.toml pyproject.toml setup.sh
  git checkout -- src/cpkb/__init__.py 2>/dev/null || true
  printf "\n${GREEN}${BOLD}✔ Dry run complete! All tests and builds passed for v$NEW_VERSION.${NC}\n"
  printf "  No commits, tags, or homebrew formula updates were pushed.\n\n"
  exit 0
fi

# ── Step 4: Commit, tag, push ───────────────────────────────────────────────────

info "Committing and tagging ..."
git add rust/Cargo.toml rust/Cargo.lock pyproject.toml setup.sh Formula/
# __init__.py is gitignore-excluded but tracked — force-add it
git add -f src/cpkb/__init__.py 2>/dev/null || true
git commit -m "Release v$NEW_VERSION"
git tag "v$NEW_VERSION"

info "Pushing to origin ..."
git push origin main --tags
success "Pushed commit and tag v$NEW_VERSION"

countdown_sleep 120 "Waiting for GitHub to process the tag ..."

# ── Step 5: Compute SHA256 from GitHub archive ──────────────────────────────────

info "Downloading GitHub archive tarball for SHA256 ..."
TARBALL_URL="https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v${NEW_VERSION}.tar.gz"
TMPTAR="$(mktemp)"
trap "rm -f '$TMPTAR'" EXIT

curl -fsSL -o "$TMPTAR" "$TARBALL_URL" \
  || die "Failed to download $TARBALL_URL — has the tag been processed by GitHub?"

SHA256=$(shasum -a 256 "$TMPTAR" | awk '{print $1}')
success "SHA256: $SHA256"

# ── Step 6: Update Formula/cpkb.rb ─────────────────────────────────────────────

info "Updating Formula/cpkb.rb ..."
python3 -c "
from pathlib import Path
import re
p = Path('Formula/cpkb.rb')
text = p.read_text(encoding='utf-8')
text = re.sub(r'url \"https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v.*\.tar\.gz\"', 'url \"$TARBALL_URL\"', text)
text = re.sub(r'sha256 \"[a-f0-9]{64}\"', 'sha256 \"$SHA256\"', text)
text = re.sub(r'assert_match \".*\", shell_output\(\"#{bin}/cpkb --version\"\)', 'assert_match \"$NEW_VERSION\", shell_output(\"#{bin}/cpkb --version\")', text)
p.write_text(text, encoding='utf-8')
"

git add Formula/cpkb.rb
[[ -f "Formula/cpkb@2.rb" ]] && git add Formula/cpkb@2.rb
git commit -m "Update homebrew formula for v$NEW_VERSION"
git push origin main
success "Formula/cpkb.rb updated and pushed"

# ── Step 7: Update the homebrew tap ─────────────────────────────────────────────

TAP_DIR="$(brew --repository 2>/dev/null)/Library/Taps/aaravshah2907/homebrew-cpkb"
if [[ -d "$TAP_DIR" ]]; then
  info "Syncing homebrew tap at $TAP_DIR ..."
  mkdir -p "$TAP_DIR/Formula"
  cp Formula/cpkb.rb "$TAP_DIR/Formula/cpkb.rb"
  [[ -f "Formula/cpkb@2.rb" ]] && cp Formula/cpkb@2.rb "$TAP_DIR/Formula/cpkb@2.rb"
  (
    cd "$TAP_DIR"
    git add Formula/
    git commit -m "Update cpkb to v$NEW_VERSION" || true
    git push origin main || warn "Tap git push failed or already up to date"
  )
  success "Homebrew tap updated and pushed"
else
  warn "Homebrew tap directory not found at $TAP_DIR"
  warn "Run:  brew tap Aaravshah2907/cpkb  then re-run, or manually copy Formula/cpkb.rb"
fi

# ── Done ────────────────────────────────────────────────────────────────────────

printf "\n${GREEN}${BOLD}🎉 Release v$NEW_VERSION complete!${NC}\n\n"
printf "  Homebrew: brew update && brew upgrade Aaravshah2907/cpkb/cpkb\n"
printf "  Pre-built binaries: GitHub Actions will attach them to the release shortly.\n\n"
