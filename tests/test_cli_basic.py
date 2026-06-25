import importlib
import sys

def test_import_cpkb_cli():
    """Ensure the CPKB CLI module can be imported without side effects."""
    module = importlib.import_module('cpkb.cli')
    assert hasattr(module, 'main')

def test_python_version():
    """Confirm the test environment runs on Python 3.11+ as required."""
    major, minor = sys.version_info[:2]
    assert (major, minor) >= (3, 11)
