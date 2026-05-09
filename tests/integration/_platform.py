"""Platform helpers shared between conftest.py and test modules."""

import platform
from pathlib import Path


def platform_cache_subdir(root: Path) -> Path:
    """Return the subdirectory that dirs::cache_dir() resolves to under root.

    On macOS the Rust dirs crate appends Library/Caches to HOME.
    On Linux and Windows it returns root directly.
    """
    if platform.system() == "Darwin":
        return root / "Library" / "Caches"
    return root


def platform_cache_env(root: Path) -> tuple:
    """Return (env_var_name, value) so dirs::cache_dir() resolves to platform_cache_subdir(root)."""
    system = platform.system()
    if system == "Darwin":
        return ("HOME", str(root))
    if system == "Windows":
        return ("LOCALAPPDATA", str(root))
    return ("XDG_CACHE_HOME", str(root))
