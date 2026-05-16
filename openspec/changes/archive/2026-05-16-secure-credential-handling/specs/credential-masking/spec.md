## ADDED Requirements

### Requirement: Database URL password is masked in terminal output
The system SHALL replace the password segment of the database URL with `****` before displaying it in any terminal output (status command, error messages).

#### Scenario: URL with password is masked in status output
- **WHEN** the configured database URL is `postgresql://user:secret@host:5432/mydb`
- **THEN** `osmprj status` displays `postgresql://user:****@host:5432/mydb` and does NOT display `secret`

#### Scenario: URL without password is displayed unchanged
- **WHEN** the configured database URL is `postgresql://user@host:5432/mydb` (no password)
- **THEN** `osmprj status` displays the URL exactly as configured

#### Scenario: URL with password is masked in connection-failed output
- **WHEN** the database URL contains a password and the connection fails
- **THEN** the failure message displays the masked URL and does NOT display the plaintext password

### Requirement: psql tip is removed from all user-facing output
The system SHALL NOT suggest `psql "<url>"` in any status output or error help text. Instead, it SHALL instruct the user to verify their database credentials.

#### Scenario: Connection failure shows generic credential advice
- **WHEN** the database connection fails
- **THEN** the output contains advice to verify database credentials and does NOT contain the string `psql`
