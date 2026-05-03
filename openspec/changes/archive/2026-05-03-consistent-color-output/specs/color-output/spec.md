## ADDED Requirements

### Requirement: Canonical icon helpers
The system SHALL provide a `crate::output` module exporting five icon functions — `icon_success()`, `icon_error()`, `icon_warn()`, `icon_info()`, and `icon_skip()` — each returning a `console::StyledObject<&'static str>` with a fixed color and symbol.

| Function        | Symbol | Style        | Semantic meaning        |
|-----------------|--------|--------------|-------------------------|
| `icon_success`  | ✓      | green        | completed / ok          |
| `icon_error`    | ✗      | red          | failed / error          |
| `icon_warn`     | ⚠      | yellow       | warning / degraded      |
| `icon_info`     | ℹ      | dim          | informational / note    |
| `icon_skip`     | ⊙      | dim          | skipped / no-op         |

All command files SHALL import icons from `crate::output` and SHALL NOT inline `console::style()` calls for status icons.

#### Scenario: Icon functions return styled output when color is enabled
- **WHEN** color output is enabled and `output::icon_success()` is called
- **THEN** the result formats as a green `✓` character

#### Scenario: Icon functions return plain output when color is disabled
- **WHEN** `NO_COLOR` is set or `--no-color` flag is passed and `output::icon_success()` is called
- **THEN** the result formats as a plain `✓` character with no ANSI codes

### Requirement: Canonical progress bar style
The system SHALL provide a `crate::output::progress_bar_style()` function returning an `indicatif::ProgressStyle` suitable for download progress bars. All progress bars in the `sync` command SHALL use this shared style.

#### Scenario: Progress bar style is importable
- **WHEN** `output::progress_bar_style()` is called
- **THEN** it returns a valid `ProgressStyle` without panicking

### Requirement: NO_COLOR environment variable compliance
The system SHALL disable all ANSI color output when the `NO_COLOR` environment variable is present and non-empty, regardless of its value, per the [no-color.org](https://no-color.org/) specification.

#### Scenario: NO_COLOR disables color in sync output
- **WHEN** the user runs `NO_COLOR=1 osmprj sync`
- **THEN** all status icons and progress bar elements in stdout/stderr contain no ANSI escape sequences

#### Scenario: NO_COLOR disables color in themes list output
- **WHEN** the user runs `NO_COLOR=1 osmprj themes list`
- **THEN** all output contains no ANSI escape sequences

#### Scenario: Empty NO_COLOR does not disable color
- **WHEN** the `NO_COLOR` environment variable is set to an empty string (`NO_COLOR=`)
- **THEN** color output is not disabled by the env var
