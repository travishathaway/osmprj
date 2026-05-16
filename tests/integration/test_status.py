"""Integration tests for the status command."""

import pytest

try:
    import tomllib
except ImportError:
    import tomli as tomllib


@pytest.fixture(autouse=True)
def clear_osmprj_database_url(monkeypatch):
    """Prevent inherited env credentials from overriding inline test URLs."""
    monkeypatch.delenv("OSMPRJ_DATABASE_URL", raising=False)


def test_status_fails_without_project(run):
    result = run("status")
    assert result.returncode != 0


def test_status_error_mentions_project_not_found(run):
    result = run("status")
    assert "osmprj.toml not found" in result.stderr


def test_status_no_database_url_shows_not_configured(run, project):
    result = run("status", cwd=project)
    assert result.returncode == 0
    assert "not configured" in result.stdout


def test_status_no_sources_shows_hint(run, project):
    result = run("status", cwd=project)
    assert result.returncode == 0
    assert "osmprj add" in result.stdout


def test_status_with_sources_lists_them(run, project):
    run("add", "--path", "/data/a.pbf", "--name", "source-a", cwd=project)
    run("add", "--path", "/data/b.pbf", "--name", "source-b", cwd=project)
    result = run("status", cwd=project)
    assert result.returncode == 0
    assert "source-a" in result.stdout
    assert "source-b" in result.stdout


def test_status_shows_schema_names(run, project):
    run("add", "--path", "/data/a.pbf", "--name", "my-region", cwd=project)
    result = run("status", cwd=project)
    assert "my_region" in result.stdout


def test_status_bad_db_url_shows_connection_failed(run, tmp_path):
    run("init", "--db", "postgres://127.0.0.1:1/nodb", cwd=tmp_path)
    result = run("status", cwd=tmp_path)
    assert result.returncode == 0
    assert "connection failed" in result.stdout


def test_status_bad_db_url_shows_helpful_tip(run, tmp_path):
    run("init", "--db", "postgres://127.0.0.1:1/nodb", cwd=tmp_path)
    result = run("status", cwd=tmp_path)
    assert "verify your database credentials" in result.stdout


def test_add_hints_when_no_db_url(run, project):
    result = run("add", "--path", "/data/r.pbf", "--name", "myregion", cwd=project)
    assert result.returncode == 0
    assert "hint" in result.stdout or "database" in result.stdout


def test_add_fails_on_bad_db_url(run, tmp_path):
    run("init", "--db", "postgres://invalid:5432/nodb", cwd=tmp_path)
    result = run("add", "--path", "/data/r.pbf", "--name", "myregion", cwd=tmp_path)
    assert result.returncode != 0


def test_add_does_not_write_source_on_bad_db_url(run, tmp_path):
    run("init", "--db", "postgres://invalid:5432/nodb", cwd=tmp_path)
    run("add", "--path", "/data/r.pbf", "--name", "myregion", cwd=tmp_path)
    config = tomllib.loads((tmp_path / "osmprj.toml").read_text())
    assert "myregion" not in config.get("sources", {})
