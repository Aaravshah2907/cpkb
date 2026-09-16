# Releasing CPKB

## Quick Release (Recommended)

```bash
./scripts/release.sh <new-version>
```

Example:

```bash
./scripts/release.sh 3.0.1
```

The script handles everything end-to-end:

1. Bumps the version in `rust/Cargo.toml`, `pyproject.toml`, `src/cpkb/__init__.py`, and `setup.sh`
2. Runs the Rust test suite (`cargo test`) and differential parity tests
3. Builds the Rust release binary and Python distributions
4. Commits, tags (`v<version>`), and pushes to origin
5. Waits 2 minutes for GitHub to process the tag
6. Downloads the GitHub archive tarball and computes its SHA256
7. Updates `Formula/cpkb.rb` and pushes
8. Syncs the homebrew tap at `$(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb` and pushes

GitHub Actions (`release-rust.yml`) then automatically builds pre-compiled binaries for macOS (arm64 + x86_64) and Linux (x86_64) and attaches them to the GitHub Release.

---

## Manual Release

### 1. Bump versions

Edit these four files to the new version:
- `rust/Cargo.toml` — `version = "X.Y.Z"` (primary source of truth)
- `pyproject.toml` — `version = "X.Y.Z"`
- `src/cpkb/__init__.py` — `__version__ = "X.Y.Z"`
- `setup.sh` — `"app_version": "X.Y.Z"`

### 2. Test

```bash
cd rust && cargo test
python3 tests/test_differential_parity.py
```

### 3. Build

```bash
# Rust release binary
cd rust && cargo build --release

# Python wheel (for PyPI legacy distribution)
python3 -m build
```

### 4. Commit, tag, push

```bash
git add rust/Cargo.toml rust/Cargo.lock pyproject.toml src/cpkb/__init__.py setup.sh Formula/
git commit -m "Release vX.Y.Z"
git tag vX.Y.Z
git push origin main --tags
```

### 5. Update Homebrew formula

After GitHub processes the tag:

```bash
TARBALL_URL="https://github.com/Aaravshah2907/cpkb/archive/refs/tags/vX.Y.Z.tar.gz"
SHA256=$(curl -fsSL "$TARBALL_URL" | shasum -a 256 | awk '{print $1}')

# Update Formula/cpkb.rb url and sha256 fields, then:
git add Formula/cpkb.rb
git commit -m "Update homebrew formula for vX.Y.Z"
git push origin main
```

Sync the tap:

```bash
TAP_DIR="$(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb"
cp Formula/cpkb.rb "$TAP_DIR/Formula/cpkb.rb"
cd "$TAP_DIR" && git add Formula/ && git commit -m "Update cpkb to vX.Y.Z" && git push origin main
```

### 6. Verify Homebrew

```bash
brew update
brew upgrade Aaravshah2907/cpkb/cpkb
cpkb --version
brew test Aaravshah2907/cpkb/cpkb
```

---

## GitHub Actions Workflows

| Workflow | Trigger | What it does |
|---|---|---|
| `rust-ci.yml` | push/PR to main | Cargo test + build + parity tests |
| `ci.yml` | push/PR to any branch | Python legacy test suite |
| `release-rust.yml` | push tag `v3.*` | Builds macOS arm64, macOS x86_64, Linux x86_64 binaries, uploads to GitHub Release |
| `publish.yml` | push tag `v*.*.*` | Builds Python wheel and publishes to PyPI (legacy) |
