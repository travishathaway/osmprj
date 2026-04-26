## ADDED Requirements

### Requirement: CLI entry point exists and is invocable
The system SHALL provide an `osmprj` console script registered in `pyproject.toml` that is executable after `pixi install`.

#### Scenario: Help output
- **WHEN** the user runs `osmprj --help`
- **THEN** the system prints usage information including all available sub-commands and exits with code 0

#### Scenario: Version output
- **WHEN** the user runs `osmprj --version`
- **THEN** the system prints the package version string and exits with code 0

### Requirement: init command placeholder
The system SHALL provide an `osmprj init` sub-command that creates an `osmprj.toml` project file in the current working directory.

#### Scenario: Init creates config file
- **WHEN** the user runs `osmprj init` in a directory without an existing `osmprj.toml`
- **THEN** the system creates an `osmprj.toml` file with default configuration and prints a confirmation message

#### Scenario: Init help
- **WHEN** the user runs `osmprj init --help`
- **THEN** the system prints usage for the init command and exits with code 0

### Requirement: add command placeholder
The system SHALL provide an `osmprj add` sub-command that records a new OSM data source in the project's `osmprj.toml`.

#### Scenario: Add help
- **WHEN** the user runs `osmprj add --help`
- **THEN** the system prints usage for the add command and exits with code 0

### Requirement: sync command placeholder
The system SHALL provide an `osmprj sync` sub-command that synchronises OSM data sources from `osmprj.toml` to the configured PostgreSQL database.

#### Scenario: Sync help
- **WHEN** the user runs `osmprj sync --help`
- **THEN** the system prints usage for the sync command and exits with code 0

### Requirement: remove command placeholder
The system SHALL provide an `osmprj remove` sub-command that removes a data source entry from `osmprj.toml`.

#### Scenario: Remove help
- **WHEN** the user runs `osmprj remove --help`
- **THEN** the system prints usage for the remove command and exits with code 0

### Requirement: destroy command placeholder
The system SHALL provide an `osmprj destroy` sub-command that removes all OSM data from the configured database.

#### Scenario: Destroy help
- **WHEN** the user runs `osmprj destroy --help`
- **THEN** the system prints usage for the destroy command and exits with code 0
