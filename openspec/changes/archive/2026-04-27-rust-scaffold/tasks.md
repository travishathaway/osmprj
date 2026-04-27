## 1. Rust Toolchain via Pixi

- [x] 1.1 Add a `rust` feature to `pyproject.toml` under `[tool.pixi.feature.rust.dependencies]` with `rust` and `cargo` from conda-forge
- [x] 1.2 Add a `rust` environment to `[tool.pixi.environments]` using the `rust` feature
- [x] 1.3 Add a `build-rust` task under `[tool.pixi.feature.rust.tasks]` that runs `cargo build --release`

## 2. Cargo Workspace

- [x] 2.1 Create `Cargo.toml` at the repo root declaring the binary crate (`name = "osmprj"`, `edition = "2021"`)
- [x] 2.2 Add dependencies: `clap` (features: `derive`), `tokio` (features: `full`), `tokio-postgres`, `toml`, `serde` (features: `derive`)
- [x] 2.3 Add `.gitignore` entry for `target/` if not already present (or update existing `.gitignore`)

## 3. Config Module

- [x] 3.1 Create `src/config.rs` with a `Settings` struct containing `database_url: Option<String>`
- [x] 3.2 Implement `Settings::load()` that reads `osmprj.toml` from CWD via the `toml` crate (missing file is silently skipped)
- [x] 3.3 Overlay `OSMPRJ_DATABASE_URL` env var on top of file value in `Settings::load()`

## 4. Database Module

- [x] 4.1 Create `src/db.rs` with an async `connect(settings: &Settings)` function returning `Result<tokio_postgres::Client, Box<dyn std::error::Error>>`
- [x] 4.2 Return a descriptive `Err` when `settings.database_url` is `None`

## 5. CLI Entry Point

- [x] 5.1 Create `src/main.rs` with a `clap` `Cli` struct and `Commands` enum covering `Init`, `Add`, `Sync`, `Remove`, `Destroy`
- [x] 5.2 Wire each variant to print "not yet implemented" and return successfully
- [x] 5.3 Annotate `main` with `#[tokio::main]`

## 6. Smoke Test

- [x] 6.1 Run `pixi run --environment rust build-rust` and confirm the release binary is produced at `target/release/osmprj`
- [x] 6.2 Run `./target/release/osmprj --help` and verify all five sub-commands appear in output
