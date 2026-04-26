## ADDED Requirements

### Requirement: Connection context manager
The system SHALL provide a `connect()` context manager in `osmprj/db.py` that yields a `psycopg.Connection` using parameters from `Settings`.

#### Scenario: Successful connection
- **WHEN** `Settings` contains a valid `database_url` and the database is reachable
- **THEN** `connect()` yields an open `psycopg.Connection` and closes it on exit

#### Scenario: Connection closed on exit
- **WHEN** the `with connect() as conn:` block exits normally
- **THEN** the connection is closed without errors

#### Scenario: Connection closed on exception
- **WHEN** the `with connect() as conn:` block raises an exception
- **THEN** the connection is still closed before the exception propagates

### Requirement: Missing database URL raises clear error
The system SHALL raise a descriptive error when `database_url` is not configured rather than producing a cryptic psycopg traceback.

#### Scenario: No database URL configured
- **WHEN** `Settings.database_url` is `None` or empty and `connect()` is called
- **THEN** the system raises a `RuntimeError` with a message indicating the database URL is not configured
