## Why

The CLI produces colored output inconsistently across commands — `sync` uses colored status icons while `status` uses bare Unicode symbols for the same semantic indicators — and does not respect the `NO_COLOR` environment variable as recommended by [no-color.org](https://no-color.org/). Users who need plain output (CI environments, accessibility tools, terminal emulators without color support) have no standard way to disable it.

## What Changes

- Add a `color-output` module (`src/output.rs`) as the single source of truth for all styled icons and shared progress bar style
- Add global `--color` / `--no-color` CLI flags; `--no-color` and the `NO_COLOR` env var both disable ANSI color, with CLI flags taking precedence
- Fix `status.rs` to use colored indicators (`✓` green, `✗` red) matching the conventions already used in `sync.rs`
- Refactor `sync.rs` and `themes.rs` to import icons from `output.rs` instead of inlining `style()` calls
- Document developer color conventions in CLAUDE.md

## Capabilities

### New Capabilities

- `color-output`: Centralized colored output module providing canonical icon helpers (`icon_success`, `icon_error`, `icon_warn`, `icon_info`, `icon_skip`), a shared `ProgressStyle` template, and NO_COLOR / CLI flag handling at startup

### Modified Capabilities

- `rust-cli`: Global CLI arguments gain `--color` / `--no-color` flags that control ANSI output, taking precedence over the `NO_COLOR` env var

## Impact

- `src/output.rs`: new file
- `src/main.rs`: NO_COLOR env var check + `--color`/`--no-color` flags wired into `Cli` struct
- `src/commands/sync.rs`: icon calls replaced with `output::icon_*()`, shared `ProgressStyle` imported from `output`
- `src/commands/themes.rs`: icon calls replaced with `output::icon_*()`
- `src/commands/status.rs`: bare `✓`/`✗` replaced with `output::icon_success()`/`output::icon_error()`
- `CLAUDE.md`: developer conventions section added
- No public API changes; no new dependencies required (uses existing `console` crate)
