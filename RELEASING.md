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
   git tag v2.0.1
   git push origin main --tags
   ```

The `Publish` workflow builds the source distribution and wheel, then publishes to PyPI.

## Homebrew

The formula in `Formula/cpkb.rb` is a template. After PyPI publishing completes:

1. Download the PyPI source distribution URL listed in the formula.
2. Replace `REPLACE_WITH_PYPI_SDIST_SHA256` with:

   ```bash
   shasum -a 256 cpkb-2.0.1.tar.gz
   ```

3. Vendor Python resources with Homebrew:

   ```bash
   brew update-python-resources Formula/cpkb.rb
   brew audit --strict --online Formula/cpkb.rb
   brew install --build-from-source Formula/cpkb.rb
   brew test cpkb
   ```

4. Commit the completed formula to a tap, for example `homebrew-cpkb`, or submit it to Homebrew core once the project meets Homebrew's acceptance criteria.
