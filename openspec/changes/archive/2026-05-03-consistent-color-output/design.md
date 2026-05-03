## Context

The CLI uses three styling libraries: `console` (0.16.3) for inline text styling via `style()`, `indicatif` (0.18.4) for progress bars, and `miette` (7.6.0) for error formatting. Color calls are scattered inline across `sync.rs`, `themes.rs`, and `status.rs` with no shared abstraction.

Key findings from the audit:
- `console` 0.16.3 checks `CLICOLOR`/`CLICOLOR_FORCE` but **not** `NO_COLOR` — the main compliance gap
- `indicatif` uses `console::colors_enabled()` internally, so fixing `console` fixes progress bars too
- `miette` uses the `supports-color` crate which already checks `NO_COLOR` — already compliant
- `status.rs` uses bare `✓`/`✗` strings while `sync.rs` uses colored `style("✓").green()` for the same semantic meaning

## Goals / Non-Goals

**Goals:**
- Respect `NO_COLOR` env var (no-color.org compliance) for all `console`-based output
- Add `--color` / `--no-color` global CLI flags that take precedence over env vars
- Centralize icon helpers and progress bar style in `src/output.rs`
- Make `status.rs` color-consistent with `sync.rs` and `themes.rs`
- Document color conventions for future developers

**Non-Goals:**
- Changing the color palette or visual design
- Supporting `CLICOLOR_FORCE` (already handled by `console` crate)
- Theming or runtime color customization beyond on/off
- Touching `miette` error formatting (already compliant)

## Decisions

### Decision: Check NO_COLOR once at startup, not at every call site

Rather than wrapping every `style()` call with a conditional, call `console::set_colors_enabled(false)` and `console::set_colors_enabled_stderr(false)` once at the top of `main()` when `NO_COLOR` is set. This propagates automatically to all `style()` calls and `indicatif` template colors with no changes to command files.

**Alternative considered**: A wrapper function that checks `NO_COLOR` on every call. Rejected — verbose, error-prone, and unnecessary given `console`'s global color state.

### Decision: Add --color / --no-color as mutually exclusive global flags

Add a `ColorArgs` group to the `Cli` struct in `main.rs` with `--color` and `--no-color` as a `clap` group. Evaluate the flags after parsing, before the `NO_COLOR` env check, so CLI flags always win.

Priority order (highest to lowest):
1. `--no-color` flag → force off
2. `--color` flag → force on (call `set_colors_enabled(true)`)
3. `NO_COLOR` env var (non-empty) → off
4. `CLICOLOR=0` env var → off (already handled by `console`)
5. Default: on if stdout is a TTY

**Alternative considered**: A single `--color=auto|always|never` option. More expressive but heavier to parse and less idiomatic for Rust CLIs.

### Decision: Centralize icons in src/output.rs, not a macro

A small module with five public functions (`icon_success`, `icon_error`, `icon_warn`, `icon_info`, `icon_skip`) and one `progress_bar_style()` function. Functions return `console::StyledObject<&'static str>` so callers use them the same way they used inline `style()` calls — they just compose naturally in format strings.

**Alternative considered**: A macro like `icon!(success)`. Rejected — functions are simpler, more discoverable, and play better with rust-analyzer.

### Decision: Keep ProgressStyle definition in output.rs

The spinner and bar styles are defined once in `output.rs` and imported by `sync.rs`. This prevents style drift if `sync.rs` is ever split into multiple files, and makes the canonical style easy to find.

## Risks / Trade-offs

- **Risk**: `console::set_colors_enabled` is a global, process-wide call. If any future code spins threads that check color state before `main()` sets it, there could be a race. → Mitigation: set the flag as the very first thing in `main()`, before any async runtime starts.
- **Risk**: `StyledObject<&'static str>` return type means icons are always static strings. Dynamic icon text would require a different return type. → Acceptable: all five icons are static.
- **Trade-off**: Importing `output::icon_success()` is slightly more verbose than `style("✓").green()`. Accepted for consistency and discoverability.

## Migration Plan

All changes are internal refactors — no CLI behavior changes for users who don't set `NO_COLOR`. The only observable behavior change is that `NO_COLOR=1 osmprj sync` now produces plain output where it previously ignored the variable.

1. Create `src/output.rs` with icon helpers and progress style
2. Update `src/main.rs`: add `ColorArgs`, evaluate flags/env var, call `set_colors_enabled`
3. Update `src/commands/status.rs`: replace bare `✓`/`✗` with `output::icon_success()`/`output::icon_error()`
4. Update `src/commands/themes.rs`: replace inline `style()` calls with `output::icon_*()`
5. Update `src/commands/sync.rs`: replace inline `style()` calls and import `progress_bar_style()` from `output`
6. Add developer guide to `CLAUDE.md`
7. Run `cargo clippy && cargo test` to verify

No rollback strategy needed — this is a pure refactor with no data migration.

## Open Questions

- Should `--color` force colors even when stdout is not a TTY (i.e., behave like `CLICOLOR_FORCE`)? Recommendation: yes — if the user explicitly passes `--color`, honor it.
