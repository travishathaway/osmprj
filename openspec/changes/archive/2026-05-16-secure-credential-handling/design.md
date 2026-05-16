## Context

osmprj currently reads the PostgreSQL connection URL exclusively from `database_url` in `osmprj.toml`. The field is typed `Option<String>` and used directly in all commands that require a database connection (`status`, `sync`, `add`, `remove`). There is no indirection layer — whatever is in the file is what gets passed to `tokio-postgres::connect()`.

The problem: `osmprj.toml` is a project file that users are expected to commit to version control. A plain-text password in this file is a credential leak waiting to happen.

The fix is an `effective_database_url()` method on `ProjectSettings` that implements a three-tier resolution chain. All existing command code switches to this single call site.

## Goals / Non-Goals

**Goals:**
- Allow the database URL to be sourced from `OSMPRJ_DATABASE_URL` env var (highest priority)
- Allow the database URL to be sourced from an arbitrary shell command (`database_url_command`)
- Maintain full backward compatibility — `database_url` in `osmprj.toml` continues to work exactly as before
- Document all three mechanisms clearly in the README and Quick Start guide
- No new external Rust dependencies

**Non-Goals:**
- OS keychain integration (deferred, not needed for Phase 1/2)
- `.env` file auto-loading (env vars work without it; complexity not justified)
- Partial URL injection (password-only substitution into a URL template)
- TLS/SSL support for the database connection (separate concern)

## Decisions

### Decision 1: Resolution order is env var → command → inline config

```
OSMPRJ_DATABASE_URL (env)
        │
        ▼  not set?
database_url_command (runs command, trims stdout, uses as URL)
        │
        ▼  not configured?
database_url (inline string in osmprj.toml)
        │
        ▼  not set?
None  (caller decides how to handle — existing behavior preserved)
```

**Why this order?** Env vars are the universally understood "override everything" mechanism, familiar from 12-factor app conventions. They are appropriate for CI and scripted environments. The command is a more structured opt-in that lives in the config file. The inline value is the legacy path.

**Alternative considered:** Command before env var. Rejected — env vars are the de facto override mechanism and should always win. A user running `OSMPRJ_DATABASE_URL=... osmprj sync` expects the env var to take effect regardless of what's in the file.

### Decision 2: `database_url_command` runs via the system shell

The command string is passed to `sh -c "<command>"` on Unix and `cmd /C "<command>"` on Windows, using `std::process::Command`. stdout is captured, trimmed of leading/trailing whitespace, and used as the URL.

**Why shell execution?** Real-world credential commands rely on shell features: pipes, redirection, variable expansion, `$PATH` lookup. Running through the shell makes all common tools (pass, op, aws, gpg, vault) work out of the box with no extra escaping by the user.

**Alternative considered:** `execvp`-style direct exec (splitting the string on whitespace). Rejected — breaks multi-word arguments, pipelines, and tools that need `$PATH`. Shell execution matches what Git, Docker, and AWS CLI do for their credential helpers.

**Alternative considered:** A dedicated `[credentials]` TOML section with typed fields per secret manager. Rejected — adds ongoing maintenance burden for every tool we'd want to support. The command approach is open-ended and future-proof.

### Decision 3: `effective_database_url()` is a free function / method on `ProjectSettings`, not a command-level concern

Resolution logic lives in `src/config.rs` as a method on `ProjectSettings`. Commands call `config.project.effective_database_url()?` instead of reading `config.project.database_url` directly.

**Why here?** Keeps resolution logic in one place. Commands should not each implement their own fallback chain. Placing it in `config.rs` also makes it straightforward to unit test without spinning up a database.

**Alternative considered:** A module-level free function in a new `src/credentials.rs`. Reasonable, but adds a file for logic that is directly tied to `ProjectSettings`. Can be refactored out later if it grows.

### Decision 4: Command execution is synchronous (blocking)

`std::process::Command` (blocking) is used rather than `tokio::process::Command` (async). The credential command runs once at startup, before any async work begins.

**Why?** Credential resolution happens before the first DB connection attempt, which is itself the beginning of meaningful async work. A blocking call at this point is simpler and avoids the need to be inside a Tokio runtime context during config loading. Command execution should complete in milliseconds.

### Decision 5: Error on non-zero exit or empty output

If `database_url_command` is set and the command:
- Exits with a non-zero status → error with the command's stderr included in the message
- Produces empty stdout (after trim) → error with a clear message

**Why not silently fall through to `database_url`?** If the user configured a command and it fails, silently using the inline URL (which may be absent or stale) would be a confusing, hard-to-debug failure mode. Fail loudly.

## Risks / Trade-offs

**[Risk] Shell injection via `database_url_command`** → The command is fully trusted user input from `osmprj.toml`, similar to how Makefile targets, npm scripts, and Cargo build scripts work. osmprj does not sanitize the command. This is acceptable because the file is under the user's control; it is not an attack surface from untrusted input. Document clearly that the command should be kept under the same access controls as the project file.

**[Risk] Command hangs indefinitely** → Credential helper commands are normally fast (sub-second). A timeout is not implemented in Phase 1. If a user's command hangs (e.g., waiting for a passphrase on a non-TTY), osmprj will hang. Mitigation: document that commands must be non-interactive in headless use. A configurable timeout can be added in a follow-up.

**[Risk] Windows path and shell differences** → `cmd /C` is used on Windows. Commands that work on Unix (`pass`, `gpg`) may not be available on Windows. Mitigation: the env var path (Phase 1) requires no shell at all and is fully cross-platform. Document platform-specific examples.

**[Trade-off] Blocking command execution** → Described in Decision 4. Acceptable given the call site.

## Migration Plan

No migration required. `database_url` continues to work without any changes to existing `osmprj.toml` files. Users who want the new behavior opt in by:
1. Removing (or not setting) `database_url`, and setting `OSMPRJ_DATABASE_URL` in their environment, or
2. Adding `database_url_command` to their `[project]` block.

No database schema changes. No lock file changes.

## Open Questions

- Should a `--no-credential-command` flag be added to allow bypassing the command for debugging purposes? Probably not needed initially — unsetting the env var or commenting out the config key is sufficient.
- Should the resolved URL be redacted in log output and `osmprj status` display? Currently `status` prints the URL. Worth a follow-up to ensure passwords are not echoed back in output.
