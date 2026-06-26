"""User configuration helpers for CPKB."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from . import __version__


DEFAULT_CONFIG: dict[str, Any] = {
    "config_version": 1,
    "app_version": __version__,
    "default_language": "cpp",
    "display": {
        "theme": "textual-dark",
        "accent_color": "cyan",
    },
    "snippets": {
        "max_number": 9999,
    },
    "backups": {
        "max_backups": 25,
    },
    "imports": {
        "load_cpp_cheatsheet_on_setup": False,
    },
}


def _merge_defaults(defaults: dict[str, Any], saved: dict[str, Any]) -> dict[str, Any]:
    """Return *saved* overlaid on *defaults*, preserving new nested defaults."""
    merged = defaults.copy()
    for key, value in saved.items():
        if isinstance(value, dict) and isinstance(merged.get(key), dict):
            merged[key] = _merge_defaults(merged[key], value)
        else:
            merged[key] = value
    return merged


def config_path(app_dir: Path) -> Path:
    """Return the config file path for an application data directory."""
    return app_dir / "config.json"


def load_config(app_dir: Path) -> dict[str, Any]:
    """Load config from *app_dir*, falling back to defaults when absent or invalid."""
    path = config_path(app_dir)
    if not path.exists():
        return DEFAULT_CONFIG.copy()

    try:
        saved = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return DEFAULT_CONFIG.copy()

    if not isinstance(saved, dict):
        return DEFAULT_CONFIG.copy()
    return _merge_defaults(DEFAULT_CONFIG, saved)


def save_config(app_dir: Path, config: dict[str, Any]) -> Path:
    """Persist *config* to *app_dir* and return the written path."""
    app_dir.mkdir(parents=True, exist_ok=True)
    path = config_path(app_dir)
    path.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
    return path


def max_snippets(app_dir: Path) -> int:
    """Return the configured maximum snippet count."""
    value = load_config(app_dir).get("snippets", {}).get("max_number", 9999)
    try:
        return max(1, int(value))
    except (TypeError, ValueError):
        return 9999


def max_backups(app_dir: Path) -> int:
    """Return the configured backup retention limit."""
    value = load_config(app_dir).get("backups", {}).get("max_backups", 25)
    try:
        return max(0, int(value))
    except (TypeError, ValueError):
        return 25
