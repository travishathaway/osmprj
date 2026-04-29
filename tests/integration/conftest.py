import platform
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
def run_cmd(binary):
    """Session-scoped helper that runs the osmprj binary and returns CompletedProcess.

    Pass ``check=False`` to suppress the assertion on returncode (e.g. when you
    want to capture a result that may fail and assert on it yourself).
    """
    def _run_cmd(*args, cwd, check=True):
        result = subprocess.run(
            [str(binary), *args],
            cwd=cwd,
            capture_output=True,
            text=True,
        )
        if check:
            assert result.returncode == 0, (
                f"osmprj {' '.join(args)} failed (rc={result.returncode}):\n{result.stderr}"
            )
        return result

    return _run_cmd


@pytest.fixture(scope="session")
def pg_e2e(tmpdir_factory):
    """Ephemeral PostgreSQL cluster on port 65112, managed by pg-helper."""
    tmp_data_dir = tmpdir_factory.mktemp("pg_e2e") / "pgdata"

    # Start server
    subprocess.run(
        ["pg-helper", "--data-dir", tmp_data_dir, "--port", "65112", "start"],
        check=True
    )

    yield "postgresql://postgres@localhost:65112/postgres"

    # Stop
    subprocess.run(
        ["pg-helper", "--data-dir", tmp_data_dir, "--port", "65112", "stop"],
        check=True
    )
    # Destroy
    subprocess.run(
        ["pg-helper", "--data-dir", tmp_data_dir, "--port", "65112", "destroy", "--force"],
        check=True
    )


@pytest.fixture(scope="session")
def geofabrik_cache_dir(tmp_path_factory):
    """Session-scoped fake cache root pre-populated with the test Geofabrik index.

    The internal layout mirrors what the ``dirs`` crate resolves on each platform:
      Linux/Windows : <root>/osmprj/geofabrik-index-v1.json
      macOS         : <root>/Library/Caches/osmprj/geofabrik-index-v1.json

    The returned path is the root that should be assigned to the platform env var
    (XDG_CACHE_HOME on Linux, HOME on macOS, LOCALAPPDATA on Windows).
    """
    root = tmp_path_factory.mktemp("geofabrik_cache")
    if platform.system() == "Darwin":
        cache_subdir = root / "Library" / "Caches"
    else:
        cache_subdir = root
    osmprj_dir = cache_subdir / "osmprj"
    osmprj_dir.mkdir(parents=True)
    shutil.copy(DATA_DIR / "geofabrik-index-v1.json", osmprj_dir / "geofabrik-index-v1.json")
    return root


@pytest.fixture
def run(binary, tmp_path, geofabrik_cache_dir, monkeypatch):
    system = platform.system()
    if system == "Darwin":
        monkeypatch.setenv("HOME", str(geofabrik_cache_dir))
    elif system == "Windows":
        monkeypatch.setenv("LOCALAPPDATA", str(geofabrik_cache_dir))
    else:
        monkeypatch.setenv("XDG_CACHE_HOME", str(geofabrik_cache_dir))

    def _run(*args, cwd=None):
        return subprocess.run(
            [str(binary), *args],
            cwd=cwd or tmp_path,
            capture_output=True,
            text=True,
        )

    return _run


@pytest.fixture
def project(run, tmp_path):
    """Temporary directory with osmprj already initialized."""
    result = run("init")
    assert result.returncode == 0
    return tmp_path
