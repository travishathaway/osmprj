## Context

`osmprj` manages OSM data sources by keeping three artefacts in sync: `osmprj.toml` (project config), `osmprj.lock` (download manifest), and a PostgreSQL schema per source. The `add` command already writes to all three; `remove` is its inverse but is currently a no-op stub.

The codebase uses `toml_edit` for round-trip TOML editing (preserving comments and formatting), `tokio_postgres` for async DB operations, and `miette` for rich error diagnostics. All existing commands follow a pattern of: validate inputs → load config → perform side effects → print results.

## Goals / Non-Goals

**Goals:**
- Remove a named source from `osmprj.toml`, `osmprj.lock`, and the database in one command.
- Accept multiple source names in a single invocation.
- Provide a `--dry-run` flag so users can preview the destructive actions before committing.
- Emit clear per-step output and actionable error messages.
- Fail early (before touching anything) if any named source is not present in `osmprj.toml`.

**Non-Goals:**
- Deleting the downloaded `.osm.pbf` file from disk — the data directory is shared and the file may be reused.
- Cascading removal of dependent objects outside the schema (e.g., cross-schema foreign keys are the user's responsibility).

## Decisions

### 1. Confirmation prompt before any mutation; `-f`/`--force` to bypass

Because schema drops are irreversible, the command SHALL print a summary of all planned actions (sources to remove, schemas to drop) and ask the user to confirm (`y/N`) before touching any artefact. This gates the destructive path regardless of how many sources are being removed.

`-f` / `--force` skips the prompt entirely, enabling scripted and CI use. `--dry-run` also bypasses the prompt (it makes no changes, so no confirmation is needed).

*Alternative considered:* Requiring the user to type the source name to confirm (like `git branch -D`). Rejected — too cumbersome when removing multiple sources and inconsistent with other tools in the OSM ecosystem.

### 2. Use `toml_edit` for `osmprj.toml` mutation (consistent with `add`)

The `add` command already uses `toml_edit` to preserve file formatting. `remove` will do the same: parse the raw file into a `DocumentMut`, call `sources_table.remove(name)`, and write back. This keeps diffs minimal and avoids reformatting the user's file.

*Alternative considered:* Deserialise with `serde`, mutate the struct, re-serialise — but this loses comments and key ordering.

### 2. `DROP SCHEMA … CASCADE` for database removal

Each source's data lives entirely inside its own schema (created by `osm2pgsql`). Dropping the schema with `CASCADE` removes all tables, sequences, and other objects at once without needing to enumerate them. This matches how `osm2pgsql` itself recommends cleaning up.

*Risk:* If the user has created cross-schema objects that reference this schema, CASCADE will drop them too. This is documented as a non-goal; the `--dry-run` output will warn about this.

### 4. Validate all source names before any mutation

The command will fail with `SourceNotFound` before touching `osmprj.toml`, the database, or `osmprj.lock` if any provided name is absent from `[sources]`. This prevents partial state where some sources are removed and others are not.

### 5. Best-effort lock-file removal (mirrors `add` behaviour for DB)

If the source is absent from `osmprj.lock` (e.g., it was added but never synced), silently skip lock removal — the state is already consistent.

### 6. Best-effort database removal

If `database_url` is not configured, or the DB is unreachable, print a warning and continue — the config and lock entries are still removed. This mirrors how `add` handles DB failures.

## Risks / Trade-offs

- **Destructive and irreversible**: `DROP SCHEMA … CASCADE` cannot be undone without a backup. Mitigation: `--dry-run` flag lets users preview; clear "Dropped schema" confirmation is printed only after success.
- **Schema name divergence**: If the user manually renamed or deleted the schema, `DROP SCHEMA IF EXISTS` will silently succeed (no error). The `IF EXISTS` variant is intentional.
- **Lock-file entry absent**: A source may exist in config but not in the lock (never synced). Mitigation: `LockFile::remove_source` is a no-op when the key is missing.

## Migration Plan

No migration required — this is a new command path. The stub `Commands::Remove` variant in `main.rs` is replaced with the real implementation. Existing users see no change until they invoke `osmprj remove`.
