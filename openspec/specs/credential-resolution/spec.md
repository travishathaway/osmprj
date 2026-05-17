### Requirement: Database URL resolves from environment variable first
When the `OSMPRJ_DATABASE_URL` environment variable is set, the system SHALL use its value as the database URL, ignoring any `database_url` value in `osmprj.toml`.

#### Scenario: Env var overrides inline config
- **WHEN** `OSMPRJ_DATABASE_URL` is set to `postgres://user:envpass@host/db` and `osmprj.toml` contains `database_url = "postgres://user:filepass@host/db"`
- **THEN** `effective_database_url()` returns `postgres://user:envpass@host/db`

#### Scenario: Env var absent falls through
- **WHEN** `OSMPRJ_DATABASE_URL` is not set
- **THEN** `effective_database_url()` proceeds to check `database_url`

### Requirement: Database URL falls back to inline config value
When `OSMPRJ_DATABASE_URL` is not set, the system SHALL return the value of `database_url` from `osmprj.toml`.

#### Scenario: Inline URL returned when no env var
- **WHEN** `OSMPRJ_DATABASE_URL` is not set and `database_url = "postgres://user:pass@host/db"`
- **THEN** `effective_database_url()` returns `postgres://user:pass@host/db`

#### Scenario: All sources absent returns None
- **WHEN** `OSMPRJ_DATABASE_URL` is not set and `database_url` is `None`
- **THEN** `effective_database_url()` returns `None`

### Requirement: Resolved URL is never forwarded as a raw string to subprocesses
The system SHALL parse the resolved database URL into structured components before any downstream use. The raw URL string (which may contain a password) SHALL NOT be passed as a CLI argument to any child process.

#### Scenario: Resolved URL is parsed into components
- **WHEN** `effective_database_url()` returns a URL containing host, port, user, password, and database
- **THEN** the system produces structured components (host, port, database, user, optional password) suitable for child process env injection and for masked display

#### Scenario: Status output does not expose the password from the resolved URL
- **WHEN** `OSMPRJ_DATABASE_URL` is set to `postgres://env:envpass@127.0.0.1:1/nodb`
- **THEN** `osmprj status` stdout contains `env:****` (masked) and does NOT contain `envpass`
