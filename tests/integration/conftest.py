import os
import shutil
import subprocess
from pathlib import Path

import pytest

BINARY = Path(__file__).parents[2] / "target" / "release" / "osmprj"
DATA_DIR = Path(__file__).parents[1] / "data"


def pytest_configure(config):
    (Path(__file__).parents[2] / "reports").mkdir(exist_ok=True)


@pytest.fixture(scope="session")
def binary():
    if not BINARY.exists():
        pytest.skip(f"Binary not found at {BINARY}; run 'cargo build --release' first.")
    return BINARY


@pytest.fixture(scope="session")
def geofabrik_cache_dir(tmp_path_factory):
    """Session-scoped cache dir pre-populated with the test Geofabrik index."""
    cache_dir = tmp_path_factory.mktemp("geofabrik_cache")
    osmprj_dir = cache_dir / "osmprj"
    osmprj_dir.mkdir()
    shutil.copy(DATA_DIR / "geofabrik-index-v1.json", osmprj_dir / "geofabrik-index-v1.json")
    return cache_dir


@pytest.fixture
def run(binary, tmp_path, geofabrik_cache_dir):
    def _run(*args, cwd=None):
        env = os.environ.copy()
        env["XDG_CACHE_HOME"] = str(geofabrik_cache_dir)
        return subprocess.run(
            [str(binary), *args],
            cwd=cwd or tmp_path,
            capture_output=True,
            text=True,
            env=env,
        )

    return _run


@pytest.fixture
def project(run, tmp_path):
    """Temporary directory with osmprj already initialized."""
    result = run("init")
    assert result.returncode == 0
    return tmp_path
