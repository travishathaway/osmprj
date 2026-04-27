# Design: End-to-End Integration Tests

## Test File Layout

```
tests/integration/
  conftest.py             ← existing; extend with DB fixtures
  test_sync_e2e.py        ← new file: slow end-to-end tests
```

## Fixtures (conftest.py additions)

### `pg_e2e` (session-scoped)

Starts a fresh PostgreSQL cluster on port **65112** using `pg-helper`:

```python
@pytest.fixture(scope="session")
def pg_e2e():
    """Start a test Postgres cluster on port 65112 via pg-helper."""
    subprocess.run(["pg-helper", "create", "--port", "65112"], check=True)
    yield "postgresql://postgres@localhost:65112/postgres"
    subprocess.run(["pg-helper", "destroy", "--port", "65112"], check=True)
```

`pg-helper` is already a dev dependency in `pixi.toml`. The fixture yields the database URL string. On teardown it destroys the cluster.

### `e2e_project` (session-scoped)

Creates a temporary directory, initialises an `osmprj` project pointed at the test DB, and adds Andorra:

```python
@pytest.fixture(scope="session")
def e2e_project(tmp_path_factory, binary, pg_e2e):
    d = tmp_path_factory.mktemp("e2e")
    run_cmd(binary, "init", "--db", pg_e2e, cwd=d)
    run_cmd(binary, "add", "andorra", "--theme", "shortbread_v1", cwd=d)
    return d
```

`run_cmd` is a small helper that wraps `subprocess.run` and asserts `returncode == 0`.

## Test Structure (`test_sync_e2e.py`)

All tests are decorated with `@pytest.mark.slow`, `@pytest.mark.integration`, and `@pytest.mark.timeout(300)`. Test functions are generic and parametrized over a `SYNC_SOURCES` list — adding a new source requires only a new `pytest.param` entry, not a new function.

### Source registry

```python
SYNC_SOURCES = [
    pytest.param("andorra", "andorra", "shortbread_v1", id="andorra"),
    # pytest.param("liechtenstein", "liechtenstein", "shortbread_v1", id="liechtenstein"),
]
```

Each param carries `(geofabrik_id, schema, theme)`.

### Test 1 — First sync (import path)

```python
@pytest.mark.slow
@pytest.mark.integration
@pytest.mark.timeout(300)
@pytest.mark.parametrize("geofabrik_id,schema,theme", SYNC_SOURCES)
def test_first_sync_imports_source(binary, e2e_project, pg_e2e, geofabrik_id, schema, theme):
    result = run_cmd(binary, "sync", cwd=e2e_project)
    assert result.returncode == 0
    with psycopg.connect(pg_e2e) as conn:
        row = conn.execute(
            f"SELECT value FROM {schema}.osm2pgsql_properties WHERE property = 'updatable'"
        ).fetchone()
    assert row is not None and row[0] == "true"
```

### Test 2 — Second sync (update path)

```python
@pytest.mark.slow
@pytest.mark.integration
@pytest.mark.timeout(300)
@pytest.mark.parametrize("geofabrik_id,schema,theme", SYNC_SOURCES)
def test_second_sync_runs_update(binary, e2e_project, pg_e2e, geofabrik_id, schema, theme):
    result = run_cmd(binary, "sync", cwd=e2e_project)
    assert result.returncode == 0
    assert "updated" in result.stdout
```

Re-uses the same session-scoped project so DB state from test 1 is preserved.

### Test 3 — Replication timestamp advances (xfail)

```python
@pytest.mark.slow
@pytest.mark.integration
@pytest.mark.timeout(300)
@pytest.mark.xfail(reason="depends on replication server having newer data")
@pytest.mark.parametrize("geofabrik_id,schema,theme", SYNC_SOURCES)
def test_replication_timestamp_advances(binary, e2e_project, pg_e2e, geofabrik_id, schema, theme):
    ...
```

## Environment Variables

The test relies on `THEMEPARK_PATH` being set (for Lua wrapper generation). In the pixi `dev` environment this is set by the conda activation hook. Tests must be run inside `pixi run --environment dev pytest tests/integration/test_sync_e2e.py -m slow`.

## Running the Slow Tests

```bash
pixi run --environment dev pytest tests/integration/test_sync_e2e.py -m slow -v
```

Or exclude them from the default run with:

```bash
pytest -m "not slow"
```

## Port Choice

Port **65112** is outside the common PostgreSQL range and unlikely to conflict with other services. `pg-helper create --port 65112` creates an isolated data directory managed by pg-helper, separate from any system PostgreSQL.

## Timeout

Individual tests are guarded with `@pytest.mark.timeout(300)` (5 minutes) via `pytest-timeout`, already in dev dependencies.
