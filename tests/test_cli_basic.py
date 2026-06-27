import importlib
import sys
from unittest.mock import patch

import pytest


def test_import_cpkb_cli():
    """Ensure the CPKB CLI module can be imported without side effects."""
    module = importlib.import_module('cpkb.cli')
    assert hasattr(module, 'main')


def test_import_cpkb_db():
    """Ensure the CPKB DB repository module can be imported."""
    module = importlib.import_module('cpkb.db')
    assert hasattr(module, 'init_db')
    assert hasattr(module, 'add_snippet')
    assert hasattr(module, 'get_snippet')


def test_import_cpkb_config():
    """Ensure the CPKB config module can be imported."""
    module = importlib.import_module('cpkb.config')
    assert hasattr(module, 'load_config')
    assert hasattr(module, 'save_config')


def test_python_version():
    """Confirm the test environment runs on Python 3.11+ as required."""
    major, minor = sys.version_info[:2]
    assert (major, minor) >= (3, 11)


def test_cli_version(capsys):
    """Ensure package managers can smoke-test the installed CLI."""
    from cpkb import __version__
    from cpkb.cli import main

    with patch.object(sys, "argv", ["cpkb", "--version"]), pytest.raises(SystemExit):
        main()

    captured = capsys.readouterr()
    assert f"cpkb {__version__}" in captured.out
