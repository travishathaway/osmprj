## 1. Create output module

- [x] 1.1 Create `src/output.rs` with `icon_success()`, `icon_error()`, `icon_warn()`, `icon_info()`, `icon_skip()` functions returning `console::StyledObject<&'static str>`
- [x] 1.2 Add `progress_bar_style()` function to `src/output.rs` returning the canonical `indicatif::ProgressStyle` for download progress bars
- [x] 1.3 Add `spinner_style()` function to `src/output.rs` returning the canonical `indicatif::ProgressStyle` for spinners
- [x] 1.4 Declare `mod output;` in `src/main.rs`

## 2. Wire up NO_COLOR and --color/--no-color flags

- [x] 2.1 Add a `clap` mutually-exclusive group (`--color` / `--no-color`) as global flags to the `Cli` struct in `src/main.rs`
- [x] 2.2 At the top of `main()`, evaluate flags then env var in priority order and call `console::set_colors_enabled()` / `console::set_colors_enabled_stderr()` accordingly
- [x] 2.3 Verify `osmprj --help` shows both `--color` and `--no-color` flags

## 3. Fix status.rs

- [x] 3.1 Import `crate::output` in `src/commands/status.rs`
- [x] 3.2 Replace bare `"✓"` with `output::icon_success()` and `"✗"` with `output::icon_error()` in the connected-database source table
- [x] 3.3 Replace bare `"✓ connected"` / `"✗ connection failed"` database-line indicators with `output::icon_success()` / `output::icon_error()`

## 4. Refactor themes.rs

- [x] 4.1 Replace `style("ℹ").dim()` with `output::icon_info()` in `src/commands/themes.rs`
- [x] 4.2 Replace `style(&entry.manifest.name).green().bold()` — keep `.bold()` inline but import the green from the canonical helper, or use `style(...).green().bold()` consistently (this is a label, not a status icon — keep inline per design)
- [x] 4.3 Remove unused `use console::style;` import if no longer needed after refactor

## 5. Refactor sync.rs

- [x] 5.1 Import `crate::output` in `src/commands/sync.rs`
- [x] 5.2 Replace `style("⊙").dim()` with `output::icon_skip()`
- [x] 5.3 Replace `style("!").yellow()` with `output::icon_warn()`
- [x] 5.4 Replace `style("✗").red()` with `output::icon_error()`
- [x] 5.5 Replace `style("✓").green()` with `output::icon_success()`
- [x] 5.6 Replace `style("⚠").yellow()` with `output::icon_warn()`
- [x] 5.7 Replace the inline `ProgressStyle::with_template(...)` bar style with `output::progress_bar_style()`
- [x] 5.8 Replace the inline spinner `ProgressStyle::with_template(...)` with `output::spinner_style()`
- [x] 5.9 Remove unused `use console::style;` import if no longer needed

## 6. Verify and document

- [x] 6.1 Run `cargo fmt && cargo clippy` — resolve any warnings
- [x] 6.2 Run `cargo test` — all tests pass
- [x] 6.3 Manual smoke test: `osmprj --no-color sync` produces no ANSI codes; `osmprj --color themes list` produces colored output
- [x] 6.4 Manual smoke test: `NO_COLOR=1 osmprj sync` produces no ANSI codes
- [x] 6.5 Manual smoke test: `NO_COLOR=1 osmprj --color themes list` produces colored output (flag overrides env var)
- [x] 6.6 Add a "Color Output Conventions" section to `CONTRIBUTING.md` documenting the icon palette, the `output.rs` module, and the rule against inlining `style()` for status icons
