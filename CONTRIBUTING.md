# Contributing to osmprj

## Color Output Conventions

All colored terminal output is centralized in `src/output.rs`. Follow these rules when adding output to any command.

### Use the canonical icon helpers

Import `crate::output` and use the five icon functions for status indicators:

| Function | Symbol | Style | When to use |
|---|---|---|---|
| `output::icon_success()` | ✓ | green | operation completed / ok |
| `output::icon_error()` | ✗ | red | operation failed / error |
| `output::icon_warn()` | ⚠ | yellow | warning / degraded / skipped with note |
| `output::icon_info()` | ℹ | dim | informational message |
| `output::icon_skip()` | ⊙ | dim | silently skipped / no-op |

```rust
use crate::output;

println!("  {} {name} imported", output::icon_success());
eprintln!("  {} {name}: {e}", output::icon_error());
```

### Never inline style() for status icons

Do **not** write `style("✓").green()` directly in command files. The icon helpers are the single source of truth — using them ensures the symbol and color stay consistent and that NO_COLOR / `--no-color` are respected automatically.

### Inline style() is fine for labels and headings

Non-icon styling (bold headings, dim paths, cyan type labels, colored counts in summary lines) can still use `console::style()` directly:

```rust
println!("  {}", style("PLUGIN THEMES").bold());
println!("  {}", style(path.display()).dim());
println!("  {} updated, {} imported", style(n_updated).green(), style(n_imported).green());
```

### Use the shared progress bar styles

For progress bars and spinners in the `sync` command (or any future command), import the shared styles from `output.rs`:

```rust
let bar_style = output::progress_bar_style();
let spinner_style = output::spinner_style();
```

### NO_COLOR and --no-color are handled automatically

`main.rs` evaluates the `NO_COLOR` env var and the `--no-color` / `--color` CLI flags at startup and calls `console::set_colors_enabled()` accordingly. You do not need to check these in individual commands — all `console::style()` calls and indicatif progress bar templates automatically respect the global color state.

Priority order (highest wins):
1. `--no-color` CLI flag → color off
2. `--color` CLI flag → color on (even when piped)
3. `NO_COLOR` env var (non-empty) → color off
4. Default: color on if stdout is a TTY
