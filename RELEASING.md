# Releasing CPKB

## Quick Release (Recommended)

Run the automated release script from the repository root:

```bash
./scripts/release.sh <new-version>
```

Example:

```bash
./scripts/release.sh 2.0.5
```

The script handles everything end-to-end:

1. Bumps the version in `pyproject.toml`, `src/cpkb/__init__.py`, and `setup.sh`
2. Runs the test suite
3. Builds the sdist and wheel
4. Commits, tags (`v<version>`), and pushes to origin
5. Downloads the GitHub archive tarball and computes its SHA256
6. Updates `Formula/cpkb.rb` in this repo and pushes
7. Syncs the homebrew tap at `$(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb` and pushes

PyPI publishing is handled automatically by the `Publish` GitHub Actions workflow on tag push.

---

## Manual Release

### PyPI

1. Create the project on PyPI as `cpkb`.
2. Configure trusted publishing for this repository and the `pypi` GitHub Actions environment.
3. Update the version in `pyproject.toml`, `src/cpkb/__init__.py`, `setup.sh`, and `Formula/cpkb.rb`.
4. Build and check locally:

   ```bash
   python3 -m pip install -e ".[dev]"
   pytest
   python3 -m build
   python3 -m twine check dist/*
   ```

5. Commit, tag, and push:

   ```bash
   git tag v<version>
   git push origin main --tags
   ```

The `Publish` workflow builds the source distribution and wheel, then publishes to PyPI.

### Homebrew

The formula in `Formula/cpkb.rb` points at the GitHub release source distribution and uses a real SHA256 checksum. Homebrew 6 requires formula developer commands to run against a tap, not an arbitrary path.

1. Create or update the tap:

   ```bash
   brew tap-new Aaravshah2907/cpkb
   cp Formula/cpkb.rb "$(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb/Formula/cpkb.rb"
   ```

2. Trust the formula while testing locally:

   ```bash
   brew trust --formula Aaravshah2907/cpkb/cpkb
   ```

3. Build the source distribution, upload it to the GitHub release as `cpkb-<version>.tar.gz`, and verify the formula's main `sha256` with that exact artifact:

   ```bash
   python3 -m build --sdist --no-isolation
   shasum -a 256 dist/cpkb-<version>.tar.gz
   ```

4. Verify the Homebrew package:

   ```bash
   brew audit --strict --online Aaravshah2907/cpkb/cpkb
   brew install --build-from-source Aaravshah2907/cpkb/cpkb
   brew test Aaravshah2907/cpkb/cpkb
   ```

5. If Homebrew edits the tap formula, copy it back:

   ```bash
   cp "$(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb/Formula/cpkb.rb" Formula/cpkb.rb
   ```

6. Commit the completed formula to the `homebrew-cpkb` tap repository, or submit it to Homebrew core once the project meets Homebrew's acceptance criteria.
