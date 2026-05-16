## ADDED Requirements

### Requirement: Child process credentials passed via environment variables, not CLI arguments
The system SHALL NOT include credentials in the `--database` / `-d` argument passed to `osm2pgsql` or `osm2pgsql-replication`. Instead, connection parameters SHALL be injected into the child process environment as `PGHOST`, `PGPORT`, `PGDATABASE`, `PGUSER`, and (if a password is present) `PGPASSWORD`.

#### Scenario: osm2pgsql receives credential-free --database argument
- **WHEN** an import is run with a database URL containing a password
- **THEN** the `osm2pgsql` process is started with `--database=postgresql://host:port/dbname` (no user or password in the argument) and `PGUSER` and `PGPASSWORD` set in its environment

#### Scenario: osm2pgsql-replication init receives credential-free -d argument
- **WHEN** replication is initialised with a database URL containing a password
- **THEN** `osm2pgsql-replication init` is called with `-d postgresql://host:port/dbname` and credentials passed via `PGUSER` / `PGPASSWORD` environment variables

#### Scenario: osm2pgsql-replication update receives credential-free -d argument
- **WHEN** a replication update is run with a database URL containing a password
- **THEN** `osm2pgsql-replication update` is called with `-d postgresql://host:port/dbname` and credentials passed via environment variables

#### Scenario: No PGPASSWORD set when URL has no password
- **WHEN** the database URL contains no password
- **THEN** `PGPASSWORD` is NOT added to the child process environment, allowing the child process to consult its own `~/.pgpass` or other libpq credential sources

### Requirement: Sync log files do not contain passwords
The `[command]` header line written to log files at the start of each subprocess invocation SHALL use the credential-free form of the command line.

#### Scenario: Log file command header contains no password
- **WHEN** a sync operation runs with a database URL containing a password
- **THEN** the log file's `[command]` line contains the credential-free URL and does NOT contain the plaintext password
