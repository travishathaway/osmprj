## 1. Config Layer

- [x] 1.1 Add `database_url_command: Option<String>` field to `ProjectSettings` in `src/config.rs`
- [x] 1.2 Add new error variants to `OsmprjError` in `src/error.rs`: `CredentialCommandFailed` (non-zero exit, includes stderr) and `CredentialCommandEmpty` (command succeeded but output was empty after trim)
- [x] 1.3 Implement `effective_database_url()` method on `ProjectSettings` in `src/config.rs` with resolution order: `OSMPRJ_DATABASE_URL` env var → `database_url_command` (shell execution) → `database_url` inline → `None`
- [x] 1.4 Write unit tests for `effective_database_url()` covering all resolution branches: env var wins, command used when no env var, inline fallback, all absent returns None, command non-zero exit errors, command empty output errors

## 2. Command Integration

- [x] 2.1 Update `src/commands/status.rs` to call `config.project.effective_database_url()?` instead of reading `config.project.database_url` directly
- [x] 2.2 Update `src/commands/sync.rs` to call `effective_database_url()?` instead of reading `database_url` directly
- [x] 2.3 Update `src/commands/add.rs` to call `effective_database_url()?` instead of reading `database_url` directly
- [x] 2.4 Update `src/commands/remove.rs` to call `effective_database_url()?` instead of reading `database_url` directly

## 3. Tests

- [x] 3.1 Add integration test: `OSMPRJ_DATABASE_URL` env var is respected and overrides `database_url` in `osmprj.toml`
- [x] 3.2 Add integration test: `database_url_command` with a simple echo command produces a working connection URL
- [x] 3.3 Add integration test: `database_url_command` with a non-zero exit code produces a clear error message
- [x] 3.4 Verify existing integration tests still pass (no regression on inline `database_url`)

## 4. Documentation

- [x] 4.1 Update `README.md` Quick Start section: replace the `osmprj init --db "postgres://user:pass@..."` example with a note warning against committing passwords, and show the env var alternative
- [x] 4.2 Update `README.md` Configuration Reference: document `database_url_command` field with description and example; add a "Credential Resolution Order" subsection explaining the three-tier hierarchy
- [x] 4.3 Update `docs/docs/getting-started.md` Quick Start: add a "Storing Credentials Securely" section after the four-command workflow that explains all three methods (env var, command, inline) with real-world `database_url_command` examples for `pass`, `op` (1Password CLI), `gpg`, and `aws secretsmanager`
- [x] 4.4 Run `cargo test` and verify all tests pass before considering the change complete
