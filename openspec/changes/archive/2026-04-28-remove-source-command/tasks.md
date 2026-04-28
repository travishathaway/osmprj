## 1. Error and DB Helpers

- [x] 1.1 Add `SourceNotFound { name: String }` variant to `OsmprjError` in `src/error.rs` with a helpful diagnostic message
- [x] 1.2 Add `drop_schema(client: &Client, schema: &str) -> Result<(), OsmprjError>` to `src/db.rs` using `DROP SCHEMA IF EXISTS "<schema>" CASCADE`

## 2. Lock File Support

- [x] 2.1 Add `remove_source(&mut self, name: &str) -> Result<(), OsmprjError>` to `LockFile` in `src/lock.rs` that removes the entry (no-op if absent) and calls `self.save()`

## 3. Remove Command Module

- [x] 3.1 Create `src/commands/remove.rs` with a `run(sources: Vec<String>, dry_run: bool, force: bool, config: &ProjectConfig) -> Result<(), OsmprjError>` function
- [x] 3.2 In `run`, validate all source names exist in `config.sources` before any mutation; return `SourceNotFound` for the first missing name
- [x] 3.3 If `--dry-run`, print planned actions for each source (config removal, lock removal, schema drop) and return without displaying a prompt or mutating anything
- [x] 3.4 Unless `--force` or `--dry-run`, print a summary of all planned removals (source names, schema names, note that the action is irreversible) and prompt `Continue? [y/N]`; abort cleanly if the user does not confirm
- [x] 3.5 For each source: remove from `osmprj.toml` using `toml_edit` (preserving file formatting), print confirmation
- [x] 3.6 For each source: attempt DB schema drop via `db::drop_schema`; warn and continue on connection/query failure; skip if no `database_url`
- [x] 3.7 For each source: call `LockFile::remove_source`; print confirmation if entry was present

## 4. CLI Wiring

- [x] 4.1 Expose `pub mod remove;` in `src/commands/mod.rs`
- [x] 4.2 Update `Commands::Remove` in `src/main.rs` to accept `sources: Vec<String>`, `#[arg(long)] dry_run: bool`, and `#[arg(short = 'f', long)] force: bool`
- [x] 4.3 Route `Commands::Remove` to `commands::remove::run(sources, dry_run, force, &config)` in the `main` match block (load `ProjectConfig` and propagate `ProjectNotFound` like other commands)

## 5. Integration Tests

- [x] 5.1 Create `tests/integration/test_remove.py`; mark the module with `pytest.mark.slow` and `pytest.mark.integration` to match the existing e2e convention
- [x] 5.2 Write a session-scoped fixture that uses `pg_e2e` and `run_cmd` to `init` a project, `add` a local-path source (no network needed), and create the schema directly via `psycopg` so the test doesn't depend on a real PBF sync
- [x] 5.3 Test `osmprj remove --force <source>` removes the entry from `osmprj.toml` (parse with `tomllib`)
- [x] 5.4 Test `osmprj remove --force <source>` removes the entry from `osmprj.lock` (parse with `tomllib`)
- [x] 5.5 Test `osmprj remove --force <source>` drops the PostgreSQL schema (query `information_schema.schemata` via `psycopg` and assert the schema is absent)
- [x] 5.6 Test `osmprj remove --dry-run <source>` leaves `osmprj.toml`, `osmprj.lock`, and the database schema unchanged
- [x] 5.7 Test `osmprj remove nonexistent` (without `--force`) exits non-zero and emits a `SourceNotFound` error mentioning the source name, without modifying any artefact
- [x] 5.8 Test `osmprj remove <source>` without `--force` prompts for confirmation (send `\n` via stdin to simulate a "no" response) and aborts cleanly without modifying any artefact

## 6. Verification

- [x] 6.1 Run `cargo build` and confirm it compiles without errors or warnings
- [x] 6.2 Run `cargo clippy` and address any lint warnings
- [x] 6.3 Manually test `osmprj remove --dry-run <source>` and confirm no prompt appears and no files are changed
- [x] 6.4 Manually test `osmprj remove <source>`: confirm the summary prompt is shown, declining aborts cleanly, confirming with `y` removes all three artefacts
- [x] 6.5 Manually test `osmprj remove -f <source>` and confirm the prompt is skipped
- [x] 6.6 Test error case: `osmprj remove nonexistent` emits a `SourceNotFound` diagnostic before any prompt
