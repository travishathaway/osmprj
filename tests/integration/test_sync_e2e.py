"""End-to-end integration tests for the full osmprj sync lifecycle.

Tests are parametrized over ``SYNC_SOURCES`` — add a new ``pytest.param`` entry
there to cover additional regions without writing new test functions.

Exclude from the default run with:
    pytest -m "not slow"
"""

import psycopg
import pytest

SYNC_SOURCES = [
    pytest.param(("monaco", "monaco", "shortbread"), id="monaco")
    # pytest.param(("liechtenstein", "liechtenstein", "shortbread"), id="liechtenstein"),
]

# ─── happy-path tests ─────────────────────────────────────────────────────────

pytestmark = [pytest.mark.slow, pytest.mark.integration, pytest.mark.timeout(300)]


@pytest.fixture(scope="module", params=SYNC_SOURCES)
def source_state(request, run_cmd_with_server, pg_e2e, tmp_path_factory):
    """Provide module fixture — one isolated osmprj project per parametrized source.

    Runs the full lifecycle: init → add → first sync → second sync, capturing
    results and replication timestamps so individual tests can assert without
    triggering additional network or DB operations.

    Module scope ensures teardown (schema removal) completes before any later
    test modules run, preventing schema state from leaking into tests that use
    the shared pg_e2e database.

    Downloads are served by the local test HTTP server (no Geofabrik traffic).
    """
    geofabrik_id, schema, theme = request.param

    project = tmp_path_factory.mktemp(f"e2e_{geofabrik_id}")
    run_cmd_with_server("init", "--db", pg_e2e, cwd=project)
    run_cmd_with_server("add", geofabrik_id, "--theme", theme, cwd=project)

    first_sync = run_cmd_with_server("sync", "--verbose", cwd=project, check=False)

    ts_before = None
    if first_sync.returncode == 0:
        with psycopg.connect(pg_e2e) as conn:
            row = conn.execute(
                f"SELECT value FROM {schema}.osm2pgsql_properties"
                " WHERE property = 'replication_timestamp'"
            ).fetchone()
            ts_before = row[0] if row else None

    second_sync = run_cmd_with_server("sync", "--verbose", cwd=project, check=False)

    ts_after = None
    if second_sync.returncode == 0:
        with psycopg.connect(pg_e2e) as conn:
            row = conn.execute(
                f"SELECT value FROM {schema}.osm2pgsql_properties"
                " WHERE property = 'replication_timestamp'"
            ).fetchone()
            ts_after = row[0] if row else None

    yield {
        "geofabrik_id": geofabrik_id,
        "schema": schema,
        "theme": theme,
        "project": project,
        "first_sync": first_sync,
        "second_sync": second_sync,
        "ts_before": ts_before,
        "ts_after": ts_after,
    }

    # Clean up
    result = run_cmd_with_server("remove", "--force", geofabrik_id, cwd=project, check=False)

    assert result.stderr == ""


def test_first_sync_imports_source(source_state, pg_e2e):
    """Ensures that the first sync was successful and created a database that can be updated."""
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
    """Ensures that the second sync ran successfully and attempted to update the database."""
    assert source_state["second_sync"].returncode == 0, (
        f"Second sync failed:\n{source_state['second_sync'].stderr}"
    )
    assert "updated" in source_state["second_sync"].stdout, (
        f"Expected 'updated' in stdout:\n{source_state['second_sync'].stdout}"
    )


def test_replication_timestamp_advances(source_state):
    """Ensure that the replication timestamp updates after each sync run."""
    ts_before = source_state["ts_before"]
    ts_after = source_state["ts_after"]
    assert ts_before is not None, "replication_timestamp not found after first sync"
    assert ts_after is not None, "replication_timestamp not found after second sync"
    assert ts_after > ts_before, (
        f"Timestamp did not advance: before={ts_before!r}, after={ts_after!r}"
    )
