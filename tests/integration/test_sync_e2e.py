"""End-to-end integration tests for the full osmprj sync lifecycle.

Tests are parametrized over ``SYNC_SOURCES`` — add a new ``pytest.param`` entry
there to cover additional regions without writing new test functions.

These tests are marked ``slow`` and ``integration``. They require:
- A running osm2pgsql binary on PATH
- ``THEMEPARK_PATH`` pointing to an osm2pgsql-themepark installation
- ``pg-helper`` available on PATH (provided by the pixi dev environment)

Run with:
    pixi run --environment dev pytest tests/integration/test_sync_e2e.py -m slow -v

Exclude from the default run with:
    pytest -m "not slow"
"""
import psycopg
import pytest

SYNC_SOURCES = [
    pytest.param(("monaco", "monaco", "shortbread_v1"), id="monaco"),
    # pytest.param(("liechtenstein", "liechtenstein", "shortbread_v1"), id="liechtenstein"),
]

pytestmark = [
    pytest.mark.slow,
    pytest.mark.integration,
    pytest.mark.timeout(300),
]


@pytest.fixture(scope="session", params=SYNC_SOURCES)
def source_state(request, run_cmd, pg_e2e, tmp_path_factory):
    """Session fixture — one isolated osmprj project per parametrized source.

    Runs the full lifecycle: init → add → first sync → second sync, capturing
    results and replication timestamps so individual tests can assert without
    triggering additional network or DB operations.
    """
    geofabrik_id, schema, theme = request.param

    project = tmp_path_factory.mktemp(f"e2e_{geofabrik_id}")
    run_cmd("init", "--db", pg_e2e, cwd=project)
    run_cmd("add", geofabrik_id, "--theme", theme, cwd=project)

    first_sync = run_cmd("sync", cwd=project, check=False)

    ts_before = None
    if first_sync.returncode == 0:
        with psycopg.connect(pg_e2e) as conn:
            row = conn.execute(
                f"SELECT value FROM {schema}.osm2pgsql_properties"
                " WHERE property = 'replication_timestamp'"
            ).fetchone()
            ts_before = row[0] if row else None

    second_sync = run_cmd("sync", cwd=project, check=False)

    ts_after = None
    if second_sync.returncode == 0:
        with psycopg.connect(pg_e2e) as conn:
            row = conn.execute(
                f"SELECT value FROM {schema}.osm2pgsql_properties"
                " WHERE property = 'replication_timestamp'"
            ).fetchone()
            ts_after = row[0] if row else None

    return {
        "geofabrik_id": geofabrik_id,
        "schema": schema,
        "theme": theme,
        "project": project,
        "first_sync": first_sync,
        "second_sync": second_sync,
        "ts_before": ts_before,
        "ts_after": ts_after,
    }


def test_first_sync_imports_source(source_state, pg_e2e):
    assert source_state["first_sync"].returncode == 0, (
        f"First sync failed:\n{source_state['first_sync'].stderr}"
    )
    schema = source_state["schema"]
    with psycopg.connect(pg_e2e) as conn:
        row = conn.execute(
            f"SELECT value FROM {schema}.osm2pgsql_properties WHERE property = 'updatable'"
        ).fetchone()
    assert row is not None, f"osm2pgsql_properties missing in schema '{schema}'"
    assert row[0] == "true", f"Expected updatable='true', got {row[0]!r}"


def test_second_sync_runs_update(source_state):
    assert source_state["second_sync"].returncode == 0, (
        f"Second sync failed:\n{source_state['second_sync'].stderr}"
    )
    assert "updated" in source_state["second_sync"].stdout, (
        f"Expected 'updated' in stdout:\n{source_state['second_sync'].stdout}"
    )


@pytest.mark.xfail(reason="depends on replication server having newer data than the PBF snapshot")
def test_replication_timestamp_advances(source_state):
    ts_before = source_state["ts_before"]
    ts_after = source_state["ts_after"]
    assert ts_before is not None, "replication_timestamp not found after first sync"
    assert ts_after is not None, "replication_timestamp not found after second sync"
    assert ts_after > ts_before, (
        f"Timestamp did not advance: before={ts_before!r}, after={ts_after!r}"
    )
