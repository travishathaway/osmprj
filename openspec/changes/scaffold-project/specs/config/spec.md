## ADDED Requirements

### Requirement: Settings class loads from environment variables
The system SHALL provide a `Settings` class (via `pydantic-settings`) in `osmprj/config.py` that reads configuration values from environment variables prefixed with `OSMPRJ_`.

#### Scenario: Database URL from env var
- **WHEN** the environment variable `OSMPRJ_DATABASE_URL` is set
- **THEN** `Settings().database_url` returns that value

#### Scenario: Missing required setting
- **WHEN** a required setting has no default and is not set in env or config file
- **THEN** instantiating `Settings()` raises a `pydantic_settings.ValidationError`

### Requirement: Settings class loads from osmprj.toml project file
The system SHALL read an `osmprj.toml` file from the current working directory as a configuration source, with environment variables taking precedence over file values.

#### Scenario: File values used when env absent
- **WHEN** `osmprj.toml` contains a `database_url` key and no `OSMPRJ_DATABASE_URL` env var is set
- **THEN** `Settings().database_url` returns the file value

#### Scenario: Env var overrides file value
- **WHEN** both `osmprj.toml` and `OSMPRJ_DATABASE_URL` env var supply a database URL
- **THEN** `Settings().database_url` returns the env var value

#### Scenario: Missing file is not an error
- **WHEN** no `osmprj.toml` file exists in the current directory
- **THEN** `Settings()` instantiates successfully using only env vars and defaults
