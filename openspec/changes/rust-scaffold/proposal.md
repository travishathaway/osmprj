## Why

A Python CLI is convenient to develop but requires a runtime and is harder to distribute to end users. A parallel Rust implementation delivers a single self-contained binary and the performance headroom needed for heavier OSM data operations, while the Python project continues as a reference implementation and development sandbox.

## What Changes

- Add `Cargo.toml` at the repository root (workspace root, single binary crate)
- Add `src/main.rs` as the Rust CLI entry point using `clap` for argument parsing
- Add placeholder sub-commands matching the Python surface: `init`, `add`, `sync`, `remove`, `destroy`
- Add a `src/config.rs` module that reads `osmprj.toml` from CWD and `OSMPRJ_*` environment variables, using the same key names as the Python `Settings` class
- Add a `src/db.rs` module with a connection helper wrapping `tokio-postgres` or `sqlx`
- Add a `rust` pixi feature that provides the Rust toolchain (`rust` from conda-forge) so `pixi run` can build and run the Rust binary without a separate `rustup` install
- Python project (`osmprj/`, `pyproject.toml`) is untouched

## Capabilities

### New Capabilities

- `rust-cli`: Rust entry point (`src/main.rs`) with `clap`-based sub-commands and `--help` / `--version`
- `rust-config`: Config module (`src/config.rs`) reading `osmprj.toml` and `OSMPRJ_*` env vars; exposes a `Settings` struct consumed by commands
- `rust-db`: Database module (`src/db.rs`) providing a connection helper that accepts a `Settings` reference and returns a live database connection

### Modified Capabilities

## Impact

- Adds `Cargo.toml` and `src/` at the repo root — no existing Python files touched
- Adds a `rust` feature to `pyproject.toml`'s `[tool.pixi.feature.*]` sections for toolchain management
- Developers wanting to build the Rust binary run `pixi run --environment rust build` (or equivalent task)
- No changes to the `osmprj` Python package API or the `osmprj.toml` config format
