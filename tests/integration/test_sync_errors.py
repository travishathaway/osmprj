"""Download error-scenario tests for osmprj sync.

These tests verify that the sync command fails cleanly and with useful error messages
when the download server returns HTTP errors, a wrong MD5, or drops the connection.

Markers: slow (need osm2pgsql on PATH).

Run with:
    pixi run --environment dev pytest tests/integration/test_sync_errors.py -m slow -v
"""

from typing import NamedTuple

import pytest

pytestmark = [pytest.mark.slow, pytest.mark.timeout(120)]

_MONACO_PBF_PATH = "/europe/monaco-latest.osm.pbf"
_MONACO_MD5_PATH = "/europe/monaco-latest.osm.pbf.md5"


class Source(NamedTuple):
    """Used to represent data sources in tests."""

    name: str
    theme: str = "shortbread"


@pytest.fixture(autouse=True)
def clear_osmprj_database_url(monkeypatch):
    """Prevent inherited env credentials from overriding test DB URLs."""
    monkeypatch.delenv("OSMPRJ_DATABASE_URL", raising=False)


def _init_error_project(run_cmd_with_server, tmp_path, sources, db_url):
    """Initialize a minimal osmprj project with the given DB URL."""
    data_dir = tmp_path / "data"
    data_dir.mkdir(parents=True)
    run_cmd_with_server("init", "--db", db_url, "--data-dir", str(data_dir), cwd=tmp_path)

    for source in sources:
        run_cmd_with_server("add", source.name, "--theme", source.theme, cwd=tmp_path)


def test_sync_503_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, pg_e2e, reset_server_errors
):
    """A 503 response from the download server causes sync to exit non-zero with a clear message."""
    _init_error_project(run_cmd_with_server, tmp_path, [Source(name="monaco")], pg_e2e)
    download_server.inject_error(_MONACO_PBF_PATH, status=503)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    combined = (result.stdout + result.stderr).lower()
    assert "503" in combined or "download" in combined, (
        f"Expected '503' or 'download' in output:\n{result.stderr}"
    )

    run_cmd_with_server("remove", "--force", "monaco", cwd=tmp_path)


def test_sync_429_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, pg_e2e, reset_server_errors
):
    """A 429 response from the download server causes sync to exit non-zero with a clear message."""
    _init_error_project(run_cmd_with_server, tmp_path, [Source(name="monaco")], pg_e2e)
    download_server.inject_error(_MONACO_PBF_PATH, status=429)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    combined = (result.stdout + result.stderr).lower()
    assert "429" in combined or "download" in combined, (
        f"Expected '429' or 'download' in output:\n{result.stderr}"
    )

    run_cmd_with_server("remove", "--force", "monaco", cwd=tmp_path)


def test_sync_404_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, pg_e2e, reset_server_errors
):
    """A 404 response (stale index) causes sync to exit non-zero and mention the status code."""
    _init_error_project(run_cmd_with_server, tmp_path, [Source(name="monaco")], pg_e2e)
    download_server.inject_error(_MONACO_PBF_PATH, status=404)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    assert "404" in (result.stdout + result.stderr), f"Expected '404' in output:\n{result.stderr}"

    run_cmd_with_server("remove", "--force", "monaco", cwd=tmp_path)


def test_sync_wrong_md5_fails_with_clear_error(
    download_server, run_cmd_with_server, tmp_path, pg_e2e, reset_server_errors
):
    """An incorrect MD5 sidecar causes sync to exit non-zero with a checksum error message."""
    _init_error_project(run_cmd_with_server, tmp_path, [Source(name="monaco")], pg_e2e)
    download_server.inject_bad_md5(_MONACO_MD5_PATH)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0
    combined = (result.stdout + result.stderr).lower()
    assert any(kw in combined for kw in ("md5", "mismatch", "checksum")), (
        f"Expected checksum error in output:\n{result.stderr}"
    )

    run_cmd_with_server("remove", "--force", "monaco", cwd=tmp_path)


def test_sync_partial_download_cleans_up(
    download_server,
    run_cmd_with_server,
    geofabrik_cache_with_server,
    tmp_path,
    pg_e2e,
    reset_server_errors,
):
    """A truncated download leaves no partial .osm.pbf file in the data directory."""
    _init_error_project(run_cmd_with_server, tmp_path, [Source(name="monaco")], pg_e2e)
    download_server.inject_partial(_MONACO_PBF_PATH, bytes_to_send=1024)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    assert result.returncode != 0

    data_dir = tmp_path / "data"
    leftover = list(data_dir.glob("*.osm.pbf")) if data_dir.exists() else []
    assert leftover == [], f"Partial .osm.pbf files were not cleaned up: {leftover}"

    run_cmd_with_server("remove", "--force", "monaco", cwd=tmp_path)


def test_first_fails_second_succeeds(
    download_server, run_cmd_with_server, tmp_path, pg_e2e, reset_server_errors
):
    """
    Ensure correct error message is displayed when the first import fails because of download
    error but the second succeeds both download and import.
    """
    _init_error_project(
        run_cmd_with_server, tmp_path, [Source(name="monaco"), Source(name="liechtenstein")], pg_e2e
    )
    download_server.inject_error(_MONACO_PBF_PATH, status=500)

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    combined = (result.stdout + result.stderr).lower()
    assert "sync failed: 1 download failed, 0 imports failed" in combined
    assert result.returncode != 0

    # Clean up
    run_cmd_with_server("remove", "--force", "liechtenstein", cwd=tmp_path)
    run_cmd_with_server("remove", "--force", "monaco", cwd=tmp_path)


def test_post_process_sql_fails(run_cmd_with_server, tmp_path, pg_e2e, reset_server_errors):
    """Ensure correct error messages appear when post-processing SQL has failed to run."""
    _init_error_project(
        run_cmd_with_server,
        tmp_path,
        [Source(name="monaco"), Source(name="liechtenstein", theme="bad-postprocess")],
        pg_e2e,
    )

    result = run_cmd_with_server("sync", cwd=tmp_path, check=False)

    # We just get a warning when the post-processing SQL fails so the process returns 0
    assert result.returncode == 0
    combined = (result.stdout + result.stderr).lower()
    assert "post-processing sql failed for source 'liechtenstein'" in combined

    # Clean up
    run_cmd_with_server("remove", "--force", "liechtenstein", cwd=tmp_path)
    run_cmd_with_server("remove", "--force", "monaco", cwd=tmp_path)
