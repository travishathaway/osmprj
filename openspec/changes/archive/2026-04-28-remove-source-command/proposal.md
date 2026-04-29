## Why

The `osmprj remove` command is a stub that prints "not yet implemented." Users who want to drop a data source from their project currently must manually edit `osmprj.toml`, delete rows from `osmprj.lock`, and run raw SQL to drop the database schema — a tedious, error-prone workflow that undermines the tool's goal of managing the full OSM project lifecycle.

## What Changes

- Implement the `osmprj remove <source>` subcommand to fully remove a named data source.
- Accept one or more source names as positional arguments.
- Remove the source entry from the `[sources]` table in `osmprj.toml` (preserving formatting via `toml_edit`).
- Drop the corresponding PostgreSQL schema (including all tables and data) from the configured database using `DROP SCHEMA … CASCADE`.
- Remove the source entry from `osmprj.lock`.
- Before making any changes, print a summary of what will be removed and prompt the user to confirm (y/N).
- Add `-f` / `--force` flag to skip the confirmation prompt for scripted or non-interactive use.
- Print a clear confirmation message for each action taken.
- Error with a user-friendly diagnostic if the named source does not exist in `osmprj.toml`.
- Add `--dry-run` flag that shows what would be removed without making any changes.

## Capabilities

### New Capabilities

- `remove-source`: Interactive CLI command that removes a named OSM data source from the project config, lock file, and database schema.

### Modified Capabilities

*(none — no existing spec-level requirements are changing)*

## Testing

Integration tests will live at `tests/integration/test_remove.py` and use the session-scoped `pg_e2e` fixture (an ephemeral PostgreSQL cluster managed by `pg-helper`) so that database-side removals are verified against a real database. The test suite will:

- Use `osmprj add` + `osmprj sync` (or direct schema creation) to seed a schema in the test database before each test.
- Run `osmprj remove` (with `--force` to bypass the interactive prompt) and assert:
  - The source entry is absent from `osmprj.toml`.
  - The source entry is absent from `osmprj.lock`.
  - The PostgreSQL schema no longer exists (`information_schema.schemata` check via `psycopg`).
- Cover the `--dry-run` path to confirm no artefact is modified.
- Cover the error path for a source name not present in `osmprj.toml`.
- Cover the confirmation-prompt path: confirm the prompt appears without `--force` and that declining leaves all artefacts unchanged (using stdin `\n` to simulate a "no" response).

These tests are marked `slow` and `integration` (matching the existing `test_sync_e2e.py` convention) and require `pg-helper` on PATH.

## Impact

- `src/main.rs`: Update `Commands::Remove` variant to accept source name(s) and route to the new command module.
- `src/commands/remove.rs`: New file implementing the remove logic.
- `src/commands/mod.rs`: Expose the new `remove` module.
- `src/db.rs`: Add `drop_schema` helper.
- `src/lock.rs`: Add `remove_source` method to `LockFile`.
- `src/error.rs`: Add `SourceNotFound` error variant.
- `osmprj.toml` / `osmprj.lock`: Modified at runtime by the command.
- `tests/integration/test_remove.py`: New integration test file using `pg_e2e`.
- No new external dependencies required.
