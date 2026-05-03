"""End-to-end integration tests for the osmprj themes system.

Parametrized over every theme in the project's ``themes/`` directory. Each
parameter runs the full lifecycle (init → add → sync) in its own isolated
osmprj project and PostgreSQL schema, then verifies that:

  1. The sync exits 0.
  2. At least one table was created in the expected schema.
  3. Every geometry column uses the SRID configured via ``--srid``.

The non-standard SRID (3505) is intentional: it exercises the full
OSMPRJ_SRID propagation path from ``osmprj add`` through the sync subprocess
into the Lua theme configuration.

Marked ``slow`` and ``integration``. Requirements:
- Release binary at ``target/release/osmprj``
- ``osm2pgsql`` and ``osm2pgsql-replication`` on PATH
- ``pg-helper`` available on PATH (provided by the pixi dev environment)
- Network access to download the Monaco PBF from Geofabrik

Run with:
    pixi run --environment dev pytest tests/integration/test_themes_e2e.py -m slow -v

Exclude from the default run with:
    pytest -m "not slow"
"""

import os
import subprocess
from pathlib import Path

import psycopg
import pytest

# Absolute path to the project's themes/ directory.
THEMES_DIR = Path(__file__).parents[2] / "themes"

# Each entry is (geofabrik_id, schema, theme_name, srid).
# Schemas are unique per theme to avoid cross-test interference in the shared
# pg_e2e cluster.  Hyphens are replaced with underscores because PostgreSQL
# schema identifiers may not contain hyphens without quoting.
# The non-standard SRID (3505) exercises the full OSMPRJ_SRID propagation path
# from ``osmprj add`` through the sync subprocess into the Lua theme config.
THEME_SOURCES = [
    pytest.param(("monaco", "pgosm", "pgosm", 3505), id="pgosm"),
    pytest.param(("monaco", "pgosm_basic", "pgosm-basic", 3505), id="pgosm-basic"),
    pytest.param(("monaco", "pgosm_minimal", "pgosm-minimal", 3505), id="pgosm-minimal"),
    pytest.param(("monaco", "pgosm_everything", "pgosm-everything", 3505), id="pgosm-everything"),
    pytest.param(("monaco", "shortbread", "shortbread", 3505), id="shortbread"),
    pytest.param(("monaco", "shortbread_gen", "shortbread-gen", 3857), id="shortbread-gen"),
    pytest.param(("monaco", "osmcarto", "osmcarto", 3505), id="osmcarto"),
    pytest.param(("monaco", "generic", "generic", 3505), id="generic"),
    pytest.param(("monaco", "nwr", "nwr", 3505), id="nwr"),
]

pytestmark = [pytest.mark.slow, pytest.mark.integration, pytest.mark.timeout(300)]


@pytest.fixture(scope="session", params=THEME_SOURCES)
def theme_state(request, binary, pg_e2e, tmp_path_factory):
    """Session fixture — one isolated osmprj project per parametrized theme.

    Runs the full lifecycle: init → add (with explicit schema and SRID) → sync,
    then queries the database so that individual test functions can assert on
    the results without triggering additional network or DB operations.
    """
    geofabrik_id, schema, theme, srid = request.param

    # Ensure the project's themes/ directory is on the search path regardless
    # of how the test runner was invoked.
    env = {**os.environ, "OSMPRJ_THEME_PATH": str(THEMES_DIR)}

    def _run(*args, cwd, check=True):
        result = subprocess.run(
            [str(binary), *args], cwd=cwd, capture_output=True, text=True, env=env, check=False
        )
        if check:
            assert result.returncode == 0, (
                f"osmprj {' '.join(str(a) for a in args)} failed "
                f"(rc={result.returncode}):\n{result.stderr}"
            )
        return result

    project = tmp_path_factory.mktemp(f"e2e_{theme.replace('-', '_')}")
    _run("init", "--db", pg_e2e, cwd=project)
    _run(
        "add", geofabrik_id, "--theme", theme, "--schema", schema, "--srid", str(srid), cwd=project
    )

    sync = _run("sync", "--verbose", cwd=project, check=False)

    tables = []
    geometry_srids = []
    if sync.returncode == 0:
        with psycopg.connect(pg_e2e) as conn:
            rows = conn.execute(
                "SELECT table_name FROM information_schema.tables"
                " WHERE table_schema = %s AND table_type = 'BASE TABLE'",
                (schema,),
            ).fetchall()
            tables = [r[0] for r in rows]

            rows = conn.execute(
                "SELECT DISTINCT srid FROM geometry_columns WHERE f_table_schema = %s", (schema,)
            ).fetchall()
            geometry_srids = [r[0] for r in rows]

    return {
        "geofabrik_id": geofabrik_id,
        "schema": schema,
        "theme": theme,
        "project": project,
        "sync": sync,
        "tables": tables,
        "geometry_srids": geometry_srids,
        "srid": srid,
    }


def test_sync_succeeds(theme_state):
    """The sync command must exit 0 for every theme."""
    assert theme_state["sync"].returncode == 0, (
        f"Sync failed for theme '{theme_state['theme']}':\n{theme_state['sync'].stderr}"
    )


def test_tables_created_in_schema(theme_state):
    """After a successful sync, at least one table must exist in the theme's schema."""
    if theme_state["sync"].returncode != 0:
        pytest.skip(f"sync failed for theme '{theme_state['theme']}'")
    assert len(theme_state["tables"]) > 0, (
        f"No tables found in schema '{theme_state['schema']}' "
        f"after syncing theme '{theme_state['theme']}'"
    )


def test_geometry_columns_use_srid(theme_state):
    """All geometry columns must carry the SRID configured via --srid."""
    if theme_state["sync"].returncode != 0:
        pytest.skip(f"sync failed for theme '{theme_state['theme']}'")
    if not theme_state["geometry_srids"]:
        pytest.skip(f"no geometry columns found in schema '{theme_state['schema']}'")
    assert all(srid == theme_state["srid"] for srid in theme_state["geometry_srids"]), (
        f"Expected all geometry columns to use SRID {theme_state['srid']}, "
        f"got {theme_state['geometry_srids']} in schema '{theme_state['schema']}' "
        f"for theme '{theme_state['theme']}'"
    )
