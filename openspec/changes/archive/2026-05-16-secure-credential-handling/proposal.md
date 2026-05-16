## Why

Currently the only way to provide a database password to osmprj is by embedding it in plain text inside `osmprj.toml`, which is a project file users are expected to commit to version control. This creates a real risk of credential exposure in git history. osmprj needs a secure, flexible credential resolution mechanism that works equally well on developer desktops and headless servers/CI environments.

## What Changes

- Add support for a `OSMPRJ_DATABASE_URL` environment variable that overrides `database_url` in `osmprj.toml` entirely.
- Add a new `database_url_command` field to `[project]` in `osmprj.toml`. When set, osmprj runs the command, reads the full database URL from stdout, and uses it in place of `database_url`.
- Define a clear resolution order: env var → command → inline config.
- Update the README Quick Start section and Configuration Reference to document the new fields and warn against committing passwords.
- Update the Docusaurus Quick Start guide (`docs/docs/getting-started.md`) with a dedicated "Storing Credentials Securely" section covering all three resolution methods with real-world examples.

## Capabilities

### New Capabilities

- `credential-resolution`: How osmprj resolves the database URL at runtime — env var override, command-based secret retrieval, and inline config fallback.

### Modified Capabilities

- `project-config`: The `ProjectSettings` struct gains a new optional field `database_url_command: Option<String>`. The existing `database_url` field is unchanged but its role becomes the last-resort fallback rather than the only option.

## Impact

- **`src/config.rs`**: Add `database_url_command` field to `ProjectSettings`. Add a method (e.g., `effective_database_url()`) that applies the resolution order and returns the resolved URL or an error.
- **`src/commands/`**: All commands that currently read `config.project.database_url` directly (`status`, `sync`, `add`, `remove`) switch to calling `effective_database_url()`.
- **`src/error.rs`**: New error variants for command execution failure and empty command output.
- **`README.md`**: Quick Start and Configuration Reference sections updated.
- **`docs/docs/getting-started.md`**: New "Storing Credentials Securely" section added.
- **No breaking changes**: `database_url` continues to work exactly as before.
- **New dependency**: None — `std::process::Command` is sufficient for running the credential command.
