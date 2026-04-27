# rust-config Specification

## Purpose

This module is the rust API for all configuration in this command line tool.

## Requirements

### Requirement: Settings struct loads from osmprj.toml
The system SHALL provide a `Settings` struct in `src/config.rs` that reads configuration from an `osmprj.toml` file in the current working directory when the file exists.

#### Scenario: database_url read from file
- **WHEN** `osmprj.toml` in the CWD contains a `database_url` key
- **THEN** `Settings::load()` returns a `Settings` with `database_url` set to that value

#### Scenario: Missing file is not an error
- **WHEN** no `osmprj.toml` exists in the CWD
- **THEN** `Settings::load()` succeeds and returns a `Settings` with `database_url` as `None`

### Requirement: Settings struct loads from environment variables
The system SHALL overlay `OSMPRJ_*` environment variables on top of file values, with env vars taking precedence.

#### Scenario: Env var overrides file value
- **WHEN** both `osmprj.toml` and `OSMPRJ_DATABASE_URL` supply a database URL
- **THEN** `Settings::load()` returns the env var value in `database_url`

#### Scenario: Env var used when no file present
- **WHEN** `OSMPRJ_DATABASE_URL` is set and no `osmprj.toml` exists
- **THEN** `Settings::load()` returns a `Settings` with `database_url` set to the env var value

