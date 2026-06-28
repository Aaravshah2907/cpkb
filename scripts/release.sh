#!/usr/bin/env bash
#
# release.sh — Automate a cpkb release
#
# Usage:
#   ./scripts/release.sh <new-version>
#
# Example:
#   ./scripts/release.sh 2.0.5
#
# This script will:
#   1. Validate the new version string
#   2. Bump the version in pyproject.toml, src/cpkb/__init__.py, setup.sh
#   3. Run the test suite
#   4. Build the sdist + wheel
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
NC=$'\033[0m' # No Color

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

# ── Pre-flight checks ──────────────────────────────────────────────────────────

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

NEW_VERSION="${1:-}"
if [[ -z "$NEW_VERSION" ]]; then
  die "Usage: $0 <new-version>  (e.g. 2.0.5)"
fi

# Validate semver-ish format
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  die "Version must be in X.Y.Z format, got: $NEW_VERSION"
fi

OLD_VERSION=$(python3 -c "
import re, pathlib
m = re.search(r'version\s*=\s*\"([^\"]+)\"', pathlib.Path('pyproject.toml').read_text())
print(m.group(1))
")
info "Current version: ${BOLD}$OLD_VERSION${NC}"
info "New version:     ${BOLD}$NEW_VERSION${NC}"

if [[ "$OLD_VERSION" == "$NEW_VERSION" ]]; then
  die "New version is the same as current version ($OLD_VERSION). Nothing to do."
fi

if git tag -l "v$NEW_VERSION" | grep -q .; then
  die "Tag v$NEW_VERSION already exists."
fi

# Ensure clean working tree (except this script itself during development)
if ! git diff --quiet HEAD; then
  warn "Working tree has uncommitted changes."
  read -rp "Continue anyway? [y/N] " confirm
  [[ "$confirm" =~ ^[Yy]$ ]] || die "Aborted."
fi

# ── Step 1: Bump version ────────────────────────────────────────────────────────

info "Bumping version from $OLD_VERSION → $NEW_VERSION ..."

# pyproject.toml
sed -i '' "s/^version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" pyproject.toml

# src/cpkb/__init__.py
sed -i '' "s/__version__ = \"$OLD_VERSION\"/__version__ = \"$NEW_VERSION\"/" src/cpkb/__init__.py

# setup.sh
sed -i '' "s/\"app_version\": \"$OLD_VERSION\"/\"app_version\": \"$NEW_VERSION\"/" setup.sh

success "Version bumped in pyproject.toml, __init__.py, setup.sh"

# ── Step 2: Run tests ───────────────────────────────────────────────────────────

info "Running test suite ..."
pytest -q
success "All tests passed"

# ── Step 3: Build sdist + wheel ─────────────────────────────────────────────────

info "Building distributions ..."
rm -rf dist/cpkb-"$NEW_VERSION"*
python3 -m build
success "Built dist/cpkb-${NEW_VERSION}.tar.gz and .whl"

# ── Step 4: Commit, tag, push ───────────────────────────────────────────────────

info "Committing and tagging ..."
git add pyproject.toml src/cpkb/__init__.py setup.sh
git commit -m "Release v$NEW_VERSION"
git tag "v$NEW_VERSION"

info "Pushing to origin ..."
git push origin main --tags
success "Pushed commit and tag v$NEW_VERSION"

countdown_sleep 120 "Waiting for package indexes and release backends to reflect the new tag ..."

# ── Step 5: Compute SHA256 from GitHub archive ──────────────────────────────────

info "Downloading GitHub archive tarball for SHA256 ..."
TARBALL_URL="https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v${NEW_VERSION}.tar.gz"
TMPTAR="$(mktemp)"
trap "rm -f '$TMPTAR'" EXIT

curl -fsSL -o "$TMPTAR" "$TARBALL_URL" \
  || die "Failed to download $TARBALL_URL — has the tag been pushed and processed by GitHub?"

SHA256=$(shasum -a 256 "$TMPTAR" | awk '{print $1}')
success "SHA256: $SHA256"

# ── Step 6: Update Formula/cpkb.rb in this repo ────────────────────────────────

info "Updating Formula/cpkb.rb ..."
sed -i '' "s|url \"https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v.*\.tar\.gz\"|url \"$TARBALL_URL\"|" Formula/cpkb.rb
sed -i '' "s/sha256 \"[a-f0-9]\{64\}\"/sha256 \"$SHA256\"/" Formula/cpkb.rb

git add Formula/cpkb.rb
git commit -m "Update homebrew formula for v$NEW_VERSION"
git push origin main
success "Formula/cpkb.rb updated and pushed"

# ── Step 7: Update the homebrew tap ─────────────────────────────────────────────

TAP_DIR="$(brew --repository 2>/dev/null)/Library/Taps/aaravshah2907/homebrew-cpkb"
if [[ -d "$TAP_DIR" ]]; then
  info "Syncing homebrew tap at $TAP_DIR ..."
  cp Formula/cpkb.rb "$TAP_DIR/Formula/cpkb.rb"
  (
    cd "$TAP_DIR"
    git add Formula/cpkb.rb
    git commit -m "Update cpkb to v$NEW_VERSION"
    git push origin main
  )
  success "Homebrew tap updated and pushed"
else
  warn "Homebrew tap directory not found at $TAP_DIR"
  warn "Run:  brew tap Aaravshah2907/cpkb  then re-run this script, or manually copy Formula/cpkb.rb"
fi

# ── Done ────────────────────────────────────────────────────────────────────────

printf "\n${GREEN}${BOLD}🎉 Release v$NEW_VERSION complete!${NC}\n\n"
printf "  PyPI:     GitHub Actions 'Publish' workflow will handle trusted publishing.\n"
printf "            Or run manually:  python3 -m twine upload dist/cpkb-${NEW_VERSION}*\n"
printf "  Homebrew: brew upgrade Aaravshah2907/cpkb/cpkb\n\n"
