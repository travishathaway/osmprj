# Tasks: End-to-End Integration Tests

## 1. Extend conftest.py

- [x] 1.1 Add a `run_cmd` helper (or reuse/adapt the existing `run` fixture) that calls the binary with `subprocess.run`, asserts `returncode == 0`, and returns the `CompletedProcess` object.
- [x] 1.2 Add a session-scoped `pg_e2e` fixture that runs `pg-helper create --port 65112`, yields the database URL `"postgresql://postgres@localhost:65112/postgres"`, and destroys the cluster on teardown with `pg-helper destroy --port 65112`.
- [x] 1.3 Add a session-scoped `e2e_project` fixture that depends on `binary`, `pg_e2e`, and `tmp_path_factory`; creates a temp dir, runs `osmprj init --db <pg_e2e_url>`, runs `osmprj add andorra --theme shortbread_v1`, and returns the project path.

## 2. Create test_sync_e2e.py

- [x] 2.1 Create `tests/integration/test_sync_e2e.py` with module-level docstring and imports (`psycopg`, `pytest`, `subprocess`, `pathlib.Path`).
- [x] 2.2 Define a `SYNC_SOURCES` list of `pytest.param` entries at module level — each entry holds `(geofabrik_id, schema, theme)`. Start with a single entry for Andorra: `pytest.param("andorra", "andorra", "shortbread_v1", id="andorra")`. New sources can be added here without writing new test functions.
- [x] 2.3 Write `test_first_sync_imports_source`: parametrized over `SYNC_SOURCES`. The fixture `e2e_project` should also be parametrized (or accept the source as an argument) so each source gets its own isolated project dir and database. After `osmprj sync`, query `<schema>.osm2pgsql_properties` via psycopg and assert `updatable = 'true'`. Decorated with `@pytest.mark.slow`, `@pytest.mark.integration`, `@pytest.mark.timeout(300)`.
- [x] 2.4 Write `test_second_sync_runs_update`: parametrized over `SYNC_SOURCES`, re-uses the same project state from task 2.3 (order guaranteed via `session` scope and explicit test ordering or a combined fixture). Asserts `returncode == 0` and `"updated"` in `result.stdout`. Same marks as 2.3.
- [x] 2.5 Write `test_replication_timestamp_advances` (xfail): parametrized over `SYNC_SOURCES`, captures `replication_timestamp` before and after a third sync call and asserts the value advanced. Decorated with `@pytest.mark.xfail(reason="depends on replication server having newer data")` in addition to slow/integration/timeout marks.

## 3. pytest.ini_options update

- [x] 3.1 Verify `pyproject.toml` already lists `"slow: ..."` under `markers` (it does). No change needed.
- [x] 3.2 Add `-m "not slow"` as a comment/note in the test README or CI configuration so slow tests are opt-in in CI — no code change if CI config doesn't exist yet.

## 4. Smoke-test the suite locally

- [ ] 4.1 Run `pixi run --environment dev pytest tests/integration/test_sync_e2e.py -m slow -v` and verify tests pass.
- [ ] 4.2 Run `pixi run --environment dev pytest -m "not slow"` and verify the new file doesn't slow down the default run.
- [ ] 4.3 Confirm `pg-helper destroy --port 65112` teardown runs cleanly after the session (check no orphan process on port 65112).
