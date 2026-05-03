# rust-db Specification

## Purpose

This module contains the database API for interacting with configured PostgreSQL databases.

## Requirements
### Requirement: Async connection helper
The system SHALL provide a `connect` async function in `src/db.rs` that accepts a `&Settings` reference and returns a live `tokio_postgres::Client`.

#### Scenario: Successful connection
- **WHEN** `settings.database_url` is set and the database is reachable
- **THEN** `connect(&settings).await` returns an `Ok(Client)` with an open connection

#### Scenario: Connection error propagated
- **WHEN** the database is unreachable or the URL is malformed
- **THEN** `connect(&settings).await` returns an `Err` containing the underlying error

### Requirement: Missing database URL returns a clear error
The system SHALL return a descriptive error when `database_url` is `None` rather than panicking or producing a cryptic message.

#### Scenario: No database URL configured
- **WHEN** `settings.database_url` is `None` and `connect` is called
- **THEN** `connect(&settings).await` returns an `Err` with a message indicating the database URL is not configured
