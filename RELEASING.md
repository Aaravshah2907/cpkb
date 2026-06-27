# Releasing CPKB

This project is set up for PyPI trusted publishing and a Homebrew formula.

## PyPI

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
   git tag v2.0.2
   git push origin main --tags
   ```

The `Publish` workflow builds the source distribution and wheel, then publishes to PyPI.

## Homebrew

The formula in `Formula/cpkb.rb` is a template until the PyPI artifact and Python resource checksums are filled in. Homebrew 6 requires formula developer commands to run against a tap, not an arbitrary path.

1. Create or update the tap:

   ```bash
   brew tap-new Aaravshah2907/cpkb
   cp Formula/cpkb.rb "$(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb/Formula/cpkb.rb"
   ```

2. Trust the formula while testing locally:

   ```bash
   brew trust --formula Aaravshah2907/cpkb/cpkb
   ```

3. Download the PyPI source distribution without dependencies and replace the formula's main `sha256` with the PyPI sdist checksum:

   ```bash
   python3 -m pip download --no-deps --no-binary :all: cpkb==2.0.2
   shasum -a 256 cpkb-2.0.2.tar.gz
   ```

4. Vendor or verify Python resources with Homebrew:

   ```bash
   brew update-python-resources Aaravshah2907/cpkb/cpkb
   brew audit --strict --online Aaravshah2907/cpkb/cpkb
   brew install --build-from-source Aaravshah2907/cpkb/cpkb
   brew test Aaravshah2907/cpkb/cpkb
   ```

5. If Homebrew edits the tap formula, copy it back:

   ```bash
   cp "$(brew --repository)/Library/Taps/aaravshah2907/homebrew-cpkb/Formula/cpkb.rb" Formula/cpkb.rb
   ```

6. Commit the completed formula to the `homebrew-cpkb` tap repository, or submit it to Homebrew core once the project meets Homebrew's acceptance criteria.
