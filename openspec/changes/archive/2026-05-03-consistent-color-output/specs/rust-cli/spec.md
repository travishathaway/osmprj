## ADDED Requirements

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
