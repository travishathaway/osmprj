### Requirement: Database URL resolves from environment variable first
When the `OSMPRJ_DATABASE_URL` environment variable is set, the system SHALL use its value as the database URL, ignoring any `database_url` or `database_url_command` values in `osmprj.toml`.

#### Scenario: Env var overrides inline config
- **WHEN** `OSMPRJ_DATABASE_URL` is set to `postgres://user:envpass@host/db` and `osmprj.toml` contains `database_url = "postgres://user:filepass@host/db"`
- **THEN** `effective_database_url()` returns `postgres://user:envpass@host/db`

#### Scenario: Env var overrides command
- **WHEN** `OSMPRJ_DATABASE_URL` is set and `osmprj.toml` contains `database_url_command = "pass show osmprj/db"`
- **THEN** `effective_database_url()` returns the env var value without executing the command

#### Scenario: Env var absent falls through
- **WHEN** `OSMPRJ_DATABASE_URL` is not set
- **THEN** `effective_database_url()` proceeds to check `database_url_command`

### Requirement: Database URL resolves from command when env var is absent
When `OSMPRJ_DATABASE_URL` is not set and `database_url_command` is configured, the system SHALL execute the command via the system shell, capture its stdout, trim leading and trailing whitespace, and use the result as the database URL.

#### Scenario: Command stdout used as URL
- **WHEN** `OSMPRJ_DATABASE_URL` is not set, `database_url_command = "echo postgres://user:cmdpass@host/db"`, and the command exits with status 0
- **THEN** `effective_database_url()` returns `postgres://user:cmdpass@host/db`

#### Scenario: Command output is trimmed
- **WHEN** the command outputs `  postgres://user:pass@host/db\n` (with surrounding whitespace)
- **THEN** `effective_database_url()` returns `postgres://user:pass@host/db` without the surrounding whitespace

#### Scenario: Command non-zero exit produces error
- **WHEN** `database_url_command` is set and the command exits with a non-zero status
- **THEN** `effective_database_url()` returns an error that includes the command's stderr output

#### Scenario: Command empty output produces error
- **WHEN** `database_url_command` is set, the command exits successfully, but stdout is empty after trimming
- **THEN** `effective_database_url()` returns an error indicating the command produced no output

#### Scenario: Command not configured falls through
- **WHEN** `OSMPRJ_DATABASE_URL` is not set and `database_url_command` is `None`
- **THEN** `effective_database_url()` proceeds to check `database_url`

### Requirement: Database URL falls back to inline config value
When neither `OSMPRJ_DATABASE_URL` nor `database_url_command` is available, the system SHALL return the value of `database_url` from `osmprj.toml`.

#### Scenario: Inline URL returned when no overrides
- **WHEN** `OSMPRJ_DATABASE_URL` is not set, `database_url_command` is `None`, and `database_url = "postgres://user:pass@host/db"`
- **THEN** `effective_database_url()` returns `postgres://user:pass@host/db`

#### Scenario: All sources absent returns None
- **WHEN** `OSMPRJ_DATABASE_URL` is not set, `database_url_command` is `None`, and `database_url` is `None`
- **THEN** `effective_database_url()` returns `None`

### Requirement: Command is executed via system shell
The system SHALL execute `database_url_command` using `sh -c` on Unix-like platforms and `cmd /C` on Windows.

#### Scenario: Unix shell execution
- **WHEN** running on a Unix platform and `database_url_command = "pass show osmprj/db | tr -d '\\n'"`
- **THEN** the command is passed to `sh -c` and pipeline operators are interpreted by the shell

#### Scenario: Windows shell execution
- **WHEN** running on Windows and `database_url_command` is set
- **THEN** the command is passed to `cmd /C`
