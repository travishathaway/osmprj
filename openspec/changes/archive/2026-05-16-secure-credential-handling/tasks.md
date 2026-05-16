## 1. Dependencies

- [x] 1.1 Add `sqlx` to `Cargo.toml` with `features = ["runtime-tokio", "postgres"]`
- [x] 1.2 Add `url` to `Cargo.toml`
- [x] 1.3 Remove `tokio-postgres` from `Cargo.toml`
- [x] 1.4 Run `cargo build` to confirm dependency changes compile

## 2. URL Utilities

- [x] 2.1 Create `src/url_utils.rs` with a `parse_db_url(url: &str) -> Result<PgConnParams, OsmprjError>` function that extracts host, port, database, user, and optional password using the `url` crate
- [x] 2.2 Implement `mask_db_url(url: &str) -> String` in `src/url_utils.rs` that replaces the password segment with `****` (returns unchanged string if no password present)
- [x] 2.3 Add `mod url_utils` to `src/main.rs` (or `lib.rs` if applicable)
- [x] 2.4 Write unit tests for `parse_db_url` covering: URL with password, URL without password, URL with no port, URL with encoded password characters
- [x] 2.5 Write unit tests for `mask_db_url` covering: URL with password, URL without password

## 3. Database Driver Migration (src/db.rs)

- [x] 3.1 Rewrite `connect(url: &str)` to use `sqlx::postgres::PgConnectOptions::from_str(url)` and `PgConnection::connect_with()`, removing the manual `tokio::spawn(connection.await)` driver task
- [x] 3.2 Rewrite `schema_exists` using `sqlx::query_scalar` or `sqlx::query().fetch_optional()`
- [x] 3.3 Rewrite `source_is_updatable` using `sqlx::query().fetch_optional()`
- [x] 3.4 Rewrite `create_schema` using `sqlx::query().execute()`
- [x] 3.5 Rewrite `drop_schema` using `sqlx::query().execute()`
- [x] 3.6 Update `use` imports — remove `tokio_postgres`, add `sqlx` types
- [x] 3.7 Run `cargo test` to confirm db module compiles and unit tests pass

## 4. Child Process Credential Isolation (src/tuner.rs + src/commands/sync.rs)

- [x] 4.1 Replace `database_url: String` field in `TunerInput` with `pg_conn: PgConnParams` (from `url_utils`)
- [x] 4.2 In `tuner::build_command`, replace `--database=<full_url>` with `--database=<credential_free_url>` (host/port/dbname only)
- [x] 4.3 Update all call sites that construct `TunerInput` to pass `PgConnParams` instead of the raw URL string
- [x] 4.4 In `sync::run_subprocess` (or its call site for the osm2pgsql invocation), inject `PGHOST`, `PGPORT`, `PGDATABASE`, `PGUSER` into the child process env via `Command::env()`; inject `PGPASSWORD` only if password is present
- [x] 4.5 In `sync::run_replication_init`, replace `-d <full_url>` with `-d <credential_free_url>` and inject the same `PG*` env vars
- [x] 4.6 In `sync::replication_update`, replace `-d <full_url>` with `-d <credential_free_url>` and inject `PG*` env vars
- [x] 4.7 Verify the `[command]` header logged to disk uses the credential-free form in all three call sites

## 5. Status Output and Error Messages

- [x] 5.1 In `src/commands/status.rs`, replace `{u}` display of URL with `mask_db_url(u)` in the connected, failed, and tip lines
- [x] 5.2 Remove the `Tip: run \`psql "{u}"\`` line from `src/commands/status.rs` (connection failure branch)
- [x] 5.3 Replace the removed tip with a message instructing the user to verify their database credentials
- [x] 5.4 In `src/error.rs`, update the `DatabaseConnectFailed` help text to remove the `psql "{url}"` tip and replace with a generic credential-verification message
- [x] 5.5 Ensure `DatabaseConnectFailed` displays the masked URL (not the raw `{url}` field) in its help text

## 6. Integration Tests

- [x] 6.1 Update `tests/integration/test_credentials.py` — change assertion `assert "env:envpass" in result.stdout` to `assert "env:****" in result.stdout`
- [x] 6.2 Add assertion `assert "envpass" not in result.stdout` in `test_env_var_overrides_inline_database_url`
- [x] 6.3 Add assertion `assert "envpass" not in result.stdout` in `test_env_var_used_when_no_inline_url`
- [x] 6.4 Run full integration test suite to confirm all tests pass

## 7. Final Verification

- [x] 7.1 Run `cargo build --release` and confirm clean build with no warnings related to unused `tokio-postgres` imports
- [x] 7.2 Run `cargo test` (all unit tests pass)
- [x] 7.3 Manual smoke test: run `osmprj status` with a URL containing a password and confirm the password is masked in output
- [x] 7.4 Manual smoke test: confirm `psql` tip no longer appears in connection failure output
