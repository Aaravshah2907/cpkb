import importlib
import sys


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


def test_python_version():
    """Confirm the test environment runs on Python 3.11+ as required."""
    major, minor = sys.version_info[:2]
    assert (major, minor) >= (3, 11)
