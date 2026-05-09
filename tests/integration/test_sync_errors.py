"""Download error-scenario tests for osmprj sync.

These tests verify that the sync command fails cleanly and with useful error messages
when the download server returns HTTP errors, a wrong MD5, or drops the connection.

Unlike test_sync_e2e.py, these tests do NOT require a database. The binary gracefully
handles a failed DB connection by treating all sources as fresh and proceeding to
download, which is where the errors are injected.

Markers: slow (need osm2pgsql on PATH), NOT integration (no DB required).

Run with:
    pixi run --environment dev pytest tests/integration/test_sync_errors.py -m slow -v
"""

import pytest
from _platform import platform_cache_subdir

pytestmark = [pytest.mark.slow, pytest.mark.timeout(120)]

_MONACO_PBF_PATH = "/europe/monaco-latest.osm.pbf"
_MONACO_MD5_PATH = "/europe/monaco-latest.osm.pbf.md5"


def _init_error_project(run_cmd_with_server, tmp_path):
    """Initialise a minimal osmprj project with a dummy (unreachable) DB URL."""
    run_cmd_with_server("init", "--db", "postgresql://localhost/nonexistent", cwd=tmp_path)
    run_cmd_with_server("add", "monaco", "--theme", "shortbread", cwd=tmp_path)


def test_sync_503_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, reset_server_errors
):
    """A 503 response from the download server causes sync to exit non-zero with a clear message."""
    _init_error_project(run_cmd_with_server, tmp_path)
    download_server.inject_error(_MONACO_PBF_PATH, status=503)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    combined = (result.stdout + result.stderr).lower()
    assert "503" in combined or "download" in combined, (
        f"Expected '503' or 'download' in output:\n{result.stderr}"
    )


def test_sync_429_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, reset_server_errors
):
    """A 429 response from the download server causes sync to exit non-zero with a clear message."""
    _init_error_project(run_cmd_with_server, tmp_path)
    download_server.inject_error(_MONACO_PBF_PATH, status=429)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    combined = (result.stdout + result.stderr).lower()
    assert "429" in combined or "download" in combined, (
        f"Expected '429' or 'download' in output:\n{result.stderr}"
    )


def test_sync_404_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, reset_server_errors
):
    """A 404 response (stale index) causes sync to exit non-zero and mention the status code."""
    _init_error_project(run_cmd_with_server, tmp_path)
    download_server.inject_error(_MONACO_PBF_PATH, status=404)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    assert "404" in (result.stdout + result.stderr), f"Expected '404' in output:\n{result.stderr}"


def test_sync_wrong_md5_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, reset_server_errors
):
    """An incorrect MD5 sidecar causes sync to exit non-zero with a checksum error message."""
    _init_error_project(run_cmd_with_server, tmp_path)
    download_server.inject_bad_md5(_MONACO_MD5_PATH)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    combined = (result.stdout + result.stderr).lower()
    assert any(kw in combined for kw in ("md5", "mismatch", "checksum")), (
        f"Expected checksum error in output:\n{result.stderr}"
    )


def test_sync_partial_download_cleans_up(
    download_server, run_cmd_with_server, geofabrik_cache_with_server, tmp_path, reset_server_errors
):
    """A truncated download leaves no partial .osm.pbf file in the data directory."""
    _init_error_project(run_cmd_with_server, tmp_path)
    download_server.inject_partial(_MONACO_PBF_PATH, bytes_to_send=1024)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0

    data_dir = platform_cache_subdir(geofabrik_cache_with_server) / "osmprj" / "geofabrik"
    leftover = list(data_dir.glob("*.osm.pbf")) if data_dir.exists() else []
    assert leftover == [], f"Partial .osm.pbf files were not cleaned up: {leftover}"
