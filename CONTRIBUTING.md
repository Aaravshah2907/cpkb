# Contributing to CPKB

Thanks for helping improve CPKB. This project is a terminal-first knowledge base for competitive programming snippets, so contributions are most useful when they keep the CLI fast, predictable, and friendly to local workflows.

## Ways to Contribute

- Report bugs with clear reproduction steps.
- Suggest commands, import/export formats, or workflow improvements.
- Improve docs, examples, tests, or optional integrations.
- Fix issues in the CLI, TUI, database layer, packaging, or Homebrew formula.

## Development Setup

```bash
git clone https://github.com/Aaravshah2907/cpkb.git
cd cpkb
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e ".[dev]"
pytest
```

For encryption-related work, the `dev` extra already installs the optional `cryptography` dependency.

## Before Opening a Pull Request

Please run:

```bash
pytest
python -m build
python -m twine check dist/*
```

If your change affects Homebrew packaging, also validate the formula from a tap:

```bash
brew audit --strict --online Aaravshah2907/cpkb/cpkb
brew install --build-from-source Aaravshah2907/cpkb/cpkb
brew test Aaravshah2907/cpkb/cpkb
```

## Pull Request Guidelines

- Keep changes focused and explain the user-facing impact.
- Add or update tests for behavior changes.
- Update README or integration docs when commands, install flows, or config change.
- Avoid committing generated artifacts such as `dist/`, local tarballs, virtual environments, or database files.

## Code of Conduct

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
