## Why

Database credentials embedded in the URL are currently passed as plaintext CLI arguments to `osm2pgsql` and `osm2pgsql-replication`, making them visible via `ps aux`, written verbatim into log files, and printed to the terminal. This exposes passwords to any user on the same machine and in stored log artifacts. Users who manage credentials via `~/.pgpass` have no supported path today because `tokio-postgres` (osmprj's current database driver) is pure Rust and does not read `.pgpass`.

## What Changes

- **Replace `tokio-postgres` with `sqlx`** (tokio runtime, postgres driver) as the database client for osmprj's own connections. `sqlx::PgConnectOptions` natively reads `~/.pgpass`, `$PGPASSFILE`, `PGPASSWORD`, `PGUSER`, `PGHOST`, `PGPORT`, and `PGDATABASE` — eliminating the need to implement pgpass parsing manually.
- **Strip credentials from child process arguments.** Instead of passing `--database=postgresql://user:secret@host/db` to `osm2pgsql` and `osm2pgsql-replication`, parse the URL into components and inject `PGHOST`, `PGPORT`, `PGDATABASE`, `PGUSER`, and `PGPASSWORD` into the child process environment. The `--database` flag retains the credential-free form (e.g. `postgresql://host:5432/dbname`).
- **Mask credentials in status output.** The `osmprj status` command displays the database URL; passwords are replaced with `****` in all terminal output.
- **Remove the `psql` tip.** The help text in error messages and status output that suggests `psql "<url>"` is removed. Users are instead told to verify their database credentials.
- **Log files no longer contain credentials.** The `[command]` header written to sync log files will use the credential-free form of the command line.

## Capabilities

### New Capabilities

- `credential-masking`: Masking of passwords in terminal output (status command, error messages) using a `mask_db_url()` utility.
- `pgpass-support`: Native `.pgpass` / `$PGPASSFILE` support for osmprj's own database connections via sqlx.
- `child-process-credential-isolation`: Passing database credentials to child processes (`osm2pgsql`, `osm2pgsql-replication`) via environment variables rather than CLI arguments.

### Modified Capabilities

- `credential-resolution`: The resolution contract (`OSMPRJ_DATABASE_URL` → `osmprj.toml`) is unchanged, but the resolved URL is now parsed into structured components before use, rather than forwarded as a raw string.
- `rust-db`: The `src/db.rs` module is rewritten to use `sqlx` instead of `tokio-postgres`. The public function signatures change; callers are updated accordingly.

## Impact

- **`Cargo.toml`**: `tokio-postgres` removed, `sqlx` added (`features = ["runtime-tokio", "postgres"]`).
- **`src/db.rs`**: Full rewrite from `tokio-postgres` to `sqlx`. All four functions (`connect`, `schema_exists`, `source_is_updatable`, `create_schema`, `drop_schema`) are updated.
- **`src/commands/status.rs`**: Credential masking applied to displayed URL; psql tip removed.
- **`src/commands/sync.rs`**: `run_replication_init` and `replication_update` inject libpq env vars instead of passing `-d <url>`; logged command lines use credential-free URL.
- **`src/tuner.rs`**: `build_command` drops `--database=<url>` arg; `TunerInput.database_url` replaced with structured connection params.
- **`src/error.rs`**: `DatabaseConnectFailed` help text updated; psql tip removed.
- **`tests/integration/test_credentials.py`**: Assertions updated — passwords must NOT appear in stdout.
