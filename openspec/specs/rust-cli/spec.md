# rust-cli Specification

## Purpose

This specification contains a high-level overview of all the subcommands available in this CLI tool.

## Requirements
### Requirement: Rust binary builds and is invocable
The system SHALL produce a `osmprj` binary via `cargo build` that is executable after a successful build.

#### Scenario: Help output
- **WHEN** the user runs `osmprj --help`
- **THEN** the binary prints usage information listing all sub-commands and exits with code 0

#### Scenario: Version output
- **WHEN** the user runs `osmprj --version`
- **THEN** the binary prints the version string from `Cargo.toml` and exits with code 0

### Requirement: init sub-command placeholder
The system SHALL provide an `osmprj init` sub-command.

#### Scenario: Init runs without error
- **WHEN** the user runs `osmprj init`
- **THEN** the binary prints a "not yet implemented" message and exits with code 0

#### Scenario: Init help
- **WHEN** the user runs `osmprj init --help`
- **THEN** the binary prints usage for init and exits with code 0

### Requirement: add sub-command placeholder
The system SHALL provide an `osmprj add` sub-command.

#### Scenario: Add runs without error
- **WHEN** the user runs `osmprj add`
- **THEN** the binary prints a "not yet implemented" message and exits with code 0

### Requirement: sync sub-command placeholder
The system SHALL provide an `osmprj sync` sub-command.

#### Scenario: Sync runs without error
- **WHEN** the user runs `osmprj sync`
- **THEN** the binary prints a "not yet implemented" message and exits with code 0

### Requirement: remove sub-command placeholder
The system SHALL provide an `osmprj remove` sub-command.

#### Scenario: Remove runs without error
- **WHEN** the user runs `osmprj remove`
- **THEN** the binary prints a "not yet implemented" message and exits with code 0

### Requirement: destroy sub-command placeholder
The system SHALL provide an `osmprj destroy` sub-command.

#### Scenario: Destroy runs without error
- **WHEN** the user runs `osmprj destroy`
- **THEN** the binary prints a "not yet implemented" message and exits with code 0

### Requirement: Global --color and --no-color flags
The system SHALL provide `--color` and `--no-color` as mutually exclusive global CLI flags available on all sub-commands. These flags SHALL take precedence over the `NO_COLOR` environment variable.

Priority order (highest to lowest):
1. `--no-color` flag → force ANSI color off
2. `--color` flag → force ANSI color on (including when stdout is not a TTY)
3. `NO_COLOR` env var (present and non-empty) → ANSI color off
4. Default: color on if stdout is a TTY, off otherwise

#### Scenario: --no-color suppresses all color output
- **WHEN** the user runs `osmprj --no-color sync`
- **THEN** all output contains no ANSI escape sequences

#### Scenario: --color forces color even when piped
- **WHEN** the user runs `osmprj --color themes list` with stdout redirected to a pipe
- **THEN** output contains ANSI color escape sequences

#### Scenario: --color overrides NO_COLOR env var
- **WHEN** the user runs `NO_COLOR=1 osmprj --color themes list`
- **THEN** output contains ANSI color escape sequences

#### Scenario: --no-color overrides CLICOLOR_FORCE
- **WHEN** the user runs `CLICOLOR_FORCE=1 osmprj --no-color sync`
- **THEN** all output contains no ANSI escape sequences

#### Scenario: Help reflects the flags
- **WHEN** the user runs `osmprj --help`
- **THEN** the help output lists both `--color` and `--no-color` flags
