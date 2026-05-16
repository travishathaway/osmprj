## MODIFIED Requirements

### Requirement: Async connection helper
The system SHALL provide a `connect` async function in `src/db.rs` that accepts a database URL `&str` and returns a live database connection. The implementation SHALL use `sqlx` with the tokio runtime and postgres driver, which natively supports `~/.pgpass`, `$PGPASSFILE`, `PGPASSWORD`, and other libpq environment variables.

#### Scenario: Successful connection
- **WHEN** the database URL is set and the database is reachable
- **THEN** `connect(url).await` returns an `Ok` connection

#### Scenario: Connection error propagated
- **WHEN** the database is unreachable or the URL is malformed
- **THEN** `connect(url).await` returns an `Err` containing the underlying error

#### Scenario: ~/.pgpass consulted when URL has no password
- **WHEN** the database URL contains no password and `~/.pgpass` has a matching entry
- **THEN** `connect(url).await` successfully authenticates using the pgpass password

### Requirement: Missing database URL returns a clear error
The system SHALL return a descriptive error when no database URL is available rather than panicking or producing a cryptic message.

#### Scenario: No database URL configured
- **WHEN** no database URL is configured and `connect` is called
- **THEN** the call returns an `Err` with a message indicating the database URL is not configured
