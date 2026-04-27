## ADDED Requirements

### Requirement: init creates osmprj.toml in the current directory
The system SHALL write an `osmprj.toml` file to the current working directory when `osmprj init` is run.

#### Scenario: Init with no options
- **WHEN** the user runs `osmprj init` in a directory without an existing `osmprj.toml`
- **THEN** the system creates `osmprj.toml` with no `database_url` set and prints a confirmation message

#### Scenario: Init with --db option
- **WHEN** the user runs `osmprj init --db postgres://user:pass@localhost/osm`
- **THEN** the system creates `osmprj.toml` with `database_url = "postgres://user:pass@localhost/osm"` and prints a confirmation message

### Requirement: init refuses to overwrite an existing osmprj.toml
The system SHALL exit with a non-zero code and a descriptive error message when `osmprj.toml` already exists in the current directory.

#### Scenario: File already exists
- **WHEN** `osmprj.toml` already exists and the user runs `osmprj init`
- **THEN** the system prints an error indicating the file already exists and exits with a non-zero code without modifying the file

### Requirement: Generated osmprj.toml contains a sources placeholder comment
The system SHALL include a comment in the generated file directing users to use `osmprj add` to register data sources.

#### Scenario: Comment present in output file
- **WHEN** `osmprj init` creates `osmprj.toml`
- **THEN** the file contains a comment referencing `osmprj add`
