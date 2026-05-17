### Requirement: osmprj's own database connection respects ~/.pgpass
The system SHALL read `~/.pgpass` (or the file specified by `$PGPASSFILE`) when establishing its own PostgreSQL connections, following the standard libpq pgpass matching rules (hostname:port:database:username:password, with `*` wildcards).

#### Scenario: Password resolved from ~/.pgpass when URL has no password
- **WHEN** the database URL contains no password (e.g. `postgresql://user@host:5432/mydb`) and `~/.pgpass` contains a matching entry
- **THEN** osmprj successfully authenticates using the password from `~/.pgpass`

#### Scenario: PGPASSFILE override is respected
- **WHEN** `$PGPASSFILE` is set to a custom path containing a matching entry and the database URL has no password
- **THEN** osmprj successfully authenticates using the password from the file at `$PGPASSFILE`

#### Scenario: Explicit password in URL takes precedence over ~/.pgpass
- **WHEN** the database URL contains an explicit password AND `~/.pgpass` has a matching entry with a different password
- **THEN** osmprj uses the password from the URL

### Requirement: osmprj's own database connection respects libpq environment variables
The system SHALL read `PGUSER`, `PGPASSWORD`, `PGHOST`, `PGPORT`, and `PGDATABASE` environment variables when establishing its own PostgreSQL connections, consistent with libpq convention.

#### Scenario: PGPASSWORD used when URL has no password
- **WHEN** `$PGPASSWORD` is set and the database URL contains no password
- **THEN** osmprj authenticates using `$PGPASSWORD`
