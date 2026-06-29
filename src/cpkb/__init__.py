"""
CPKB - Competitive Programming Knowledge Base
"""

try:
    import importlib.metadata
    __version__ = importlib.metadata.version("cpkb")
except importlib.metadata.PackageNotFoundError:
    __version__ = "unknown"
