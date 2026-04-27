# Proposal: End-to-End Integration Tests

## What

Add a `pytest` test suite under `tests/integration/` marked `@pytest.mark.slow` that exercises the full `osmprj sync` lifecycle against a real PostgreSQL database:

1. Spin up a test database on port 65112 using the `pg-helper` CLI.
2. Initialize an `osmprj` project pointed at that database.
3. Add Andorra as a source (smallest Geofabrik country, ~1 MB PBF).
4. Run `osmprj sync` — verifies download, full import, and replication init succeed.
5. Run `osmprj sync` again — verifies the update path (`osm2pgsql-replication update`) runs without error and the database is updated.

## Why

The existing integration tests (`tests/integration/`) cover CLI argument validation and error cases but stop short of actually running `osm2pgsql`. The new "update" code path (classifying sources via `osm2pgsql_properties`, running `replication_update`) has no test coverage at all. A bug there would be invisible until someone runs a real sync twice.

Running against Andorra keeps test time acceptable (~2–3 minutes total) while exercising every non-trivial code path: download, MD5 verification, Lua wrapper generation, osm2pgsql import, replication init, and replication update.

## Non-Goals

- Unit-testing individual Rust functions (covered by `cargo test`).
- Testing against a production-size dataset.
- Parallelising the slow tests (sequential is fine for now).
- Windows/macOS CI (Linux only, consistent with `pixi.toml` platform lock).
