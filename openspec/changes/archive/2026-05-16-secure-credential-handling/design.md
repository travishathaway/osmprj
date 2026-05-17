## Context

`osmprj` currently uses `tokio-postgres` (pure Rust, no libpq) as its database driver. Database credentials are stored in the URL (`postgresql://user:secret@host/db`) and forwarded as-is to child processes (`osm2pgsql`, `osm2pgsql-replication`) via CLI arguments. This creates three credential exposure vectors:

1. **`ps aux`** — plaintext password visible in argv to all users on the machine
2. **Log files** — the `[command] osm2pgsql ... --database=<url>` header written at the start of every sync embeds the full URL
3. **Terminal output** — `osmprj status` prints the raw URL including the password; verbose sync mode echoes the command line

Additionally, users who manage credentials via `~/.pgpass` cannot use that mechanism for osmprj's own connections because `tokio-postgres` does not implement pgpass parsing.

## Goals / Non-Goals

**Goals:**
- Passwords never appear in child process argv (not visible via `ps`, not logged)
- `~/.pgpass` and `$PGPASSFILE` work for osmprj's own DB connections
- `osmprj status` displays the URL with password masked as `****`
- Error messages do not expose the raw URL or suggest `psql "<url>"`
- Log files do not contain passwords
- `PGPASSWORD`, `PGUSER`, `PGHOST`, `PGPORT`, `PGDATABASE` env vars respected by osmprj itself (via sqlx) and by child processes (via inherited env)

**Non-Goals:**
- Supporting `pgpass` as a new `osmprj.toml` config key
- Changing the `OSMPRJ_DATABASE_URL` / `database_url` precedence rules
- Encrypting credentials at rest (out of scope)
- Supporting credential stores (Vault, Keychain, etc.)

## Decisions

### Decision 1: Replace `tokio-postgres` with `sqlx` for osmprj's own connections

**Choice**: `sqlx` with `features = ["runtime-tokio", "postgres"]`.

**Rationale**: `sqlx::PgConnectOptions` implements the full libpq credential resolution chain natively — `PGPASSWORD`, `PGPASSFILE`, `~/.pgpass` (including file permission checks on Unix), and all `PG*` environment variables. Implementing this manually against `tokio-postgres` would be ~100 lines of correct-but-non-standard Rust with edge cases (percent-encoded passwords, IPv6 hosts, Windows path, file permission check). The cost is higher compile times and a moderately larger binary; for a CLI tool this is acceptable.

The `src/db.rs` API surface is small (5 functions, 68 lines). Migration is mechanical: `tokio-postgres::Client` → `sqlx::PgPool` or single `PgConnection`, `client.query_opt()` → `sqlx::query().fetch_optional()`, `client.execute()` → `sqlx::query().execute()`. The manual connection driver spawn (`tokio::spawn(connection.await)`) disappears — sqlx manages this internally.

`sqlx`'s runtime query API (not the compile-time `query!` macros) is used throughout to avoid the requirement for a live database at compile time.

**Alternative considered**: Implement `.pgpass` parsing manually in `tokio-postgres`. Rejected — adds maintenance burden and non-standard behaviour. The `url` crate would also be needed for correct URL parsing, adding another dependency with no other benefit.

### Decision 2: Pass credentials to child processes via libpq environment variables

**Choice**: Parse the resolved database URL into components (`PGHOST`, `PGPORT`, `PGDATABASE`, `PGUSER`, `PGPASSWORD`) and inject them into the child process environment via `tokio::process::Command::env()`. Pass `--database=postgresql://host:port/dbname` (credential-free) as the CLI argument.

**Rationale**: `osm2pgsql` and `osm2pgsql-replication` both link libpq and read the standard `PG*` environment variables. Credentials in env vars are not visible in `ps aux` output (argv is world-readable on Linux; `/proc/<pid>/environ` is owner-readable only). The logged `[command]` line will contain only the credential-free URL form, eliminating log file exposure.

`PGPASSWORD` is set only when a password is present in the URL. If the user has no password in their URL (relying on `.pgpass` or trust auth), `PGPASSWORD` is not set and the child process falls through to its own libpq credential lookup, including `~/.pgpass`.

**Alternative considered**: Use a temp pgpass file. More secure than `PGPASSWORD` but adds complexity (temp file lifecycle, permissions, cleanup on panic). Deferred as a future hardening step.

**Alternative considered**: Pass `PGPASSFILE` to child processes. Only useful if osmprj generates a temp pgpass file (see above). Not needed for the current scope.

### Decision 3: URL parsing for credential extraction

**Choice**: Use the `url` crate to parse the database URL into components before passing to child processes and for masking display output.

**Rationale**: The `url` crate correctly handles percent-encoded passwords, IPv6 bracketed hosts, and query-string host overrides. String splitting on `://` and `@` is fragile. `sqlx` itself depends on `url` (it's already a transitive dependency after the sqlx migration), so adding it directly has zero net dependency cost.

### Decision 4: Credential masking in status output

**Choice**: A `mask_db_url(url: &str) -> String` utility in `src/output.rs` (or a new `src/url_utils.rs`) that replaces the password segment with `****`. Applied to all terminal-facing displays of the URL. The raw URL is never printed.

**Rationale**: Users need to see enough of the URL (host, port, database, user) to diagnose misconfiguration. Masking only the password preserves this. The `psql` tip (which required the unmasked URL for copy-paste) is removed; users are instead prompted to verify credentials.

## Risks / Trade-offs

- **Compile time regression** — `sqlx` is a large crate with macro machinery. Even using only runtime queries, compile times will increase. → Mitigation: use `sqlx` with only `runtime-tokio` and `postgres` features; avoid `sqlx-macros` unless explicitly needed.
- **`PGPASSWORD` env var still readable by process owner** — `/proc/<pid>/environ` on Linux is readable by the process owner. This is significantly better than argv (readable by all users) but not hermetic. → Accepted trade-off; documented in user-facing materials if needed.
- **URL parsing edge cases** — URLs with no host (Unix socket paths, `%2F`-encoded), no port, or query-string overrides need correct handling in the credential-extraction path. → Mitigated by using the `url` crate rather than string splitting.
- **`tokio-postgres` removal** — Any future code that imports `tokio_postgres` directly will break. → Impact is limited to `src/db.rs` and its callers; the change is contained.

## Migration Plan

1. Add `sqlx` and `url` to `Cargo.toml`; remove `tokio-postgres`.
2. Rewrite `src/db.rs` using `sqlx`. Keep function names stable where possible to minimise caller churn.
3. Introduce URL parsing utility (`mask_db_url`, `parse_db_url` → `PgConnParams`).
4. Update `src/tuner.rs` to accept `PgConnParams` instead of a raw URL string; drop `--database=<url>` arg.
5. Update `src/commands/sync.rs` subprocess calls to inject env vars.
6. Update `src/commands/status.rs` to use masked URL; remove psql tip.
7. Update `src/error.rs` to remove psql tip from help text.
8. Update integration tests to assert passwords do NOT appear in stdout.
9. Build and run full test suite.

Rollback: revert to `tokio-postgres` by reverting `Cargo.toml` and `src/db.rs`. All other changes (env var injection, masking) are independent and low-risk.

## Open Questions

- None — design is sufficiently clear to proceed to implementation.
