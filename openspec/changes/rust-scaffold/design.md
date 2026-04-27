## Context

The Python `osmprj` package is scaffolded and working. This change adds a parallel Rust implementation targeting the same user-facing CLI surface (`init`, `add`, `sync`, `remove`, `destroy`) and the same config contract (`osmprj.toml` + `OSMPRJ_*` env vars). The two implementations coexist in the same repository; neither replaces the other at this stage. The Rust binary is the intended long-term distribution artifact.

## Goals / Non-Goals

**Goals:**
- Standard Rust binary crate at the repo root (`Cargo.toml` + `src/`)
- `clap` (derive API) for sub-commands and `--help` / `--version`
- `config.rs` reading `osmprj.toml` via `toml` crate and env vars via `std::env`, exposing a `Settings` struct
- `db.rs` connection helper wrapping `tokio-postgres`; async runtime via `tokio`
- `rust` pixi feature providing the Rust toolchain from conda-forge so the project needs no external `rustup`
- Pixi task `build-rust` that runs `cargo build --release`

**Non-Goals:**
- PyO3 / Python bindings — Rust and Python stay as separate binaries
- Implementing actual OSM import logic in Rust (placeholder commands only)
- Cross-platform builds beyond `linux-64` (matches existing pixi platform list)

## Decisions

### CLI library: clap (derive API)

`clap` is the de-facto standard for Rust CLIs. The derive API (`#[derive(Parser, Subcommand)]`) keeps the command definition declarative and close to the Python `cyclopts` style. Generates `--help`, `--version`, and shell completion stubs automatically.

**Alternatives considered:**
- *argh*: lighter but minimal ecosystem and no shell completion
- *structopt*: superseded by clap v4

### Async runtime: tokio

`tokio-postgres` requires an async runtime. `tokio` is the standard choice and required by most Rust database drivers. The binary will use `#[tokio::main]` on `main`. At placeholder stage this is zero overhead; it's needed for the `db.rs` module to be non-trivial.

**Alternatives considered:**
- *async-std*: smaller ecosystem, fewer crates support it
- *Synchronous postgres crate*: would require switching drivers when async work is needed later; not worth the migration cost

### Database driver: tokio-postgres

Direct, low-level driver. Mirrors the Python choice of raw `psycopg` over an ORM. Keeps the dependency graph lean.

**Alternatives considered:**
- *sqlx*: compile-time query checking is valuable but requires a live database at build time, which complicates CI
- *diesel*: ORM-level abstraction; not needed for a CLI tool at this stage

### Config: toml crate + std::env (no config framework)

At scaffold stage, `Settings` has one field (`database_url`). A full config framework (`config-rs`, `figment`) adds complexity not yet warranted. A thin hand-rolled loader — parse `osmprj.toml` with the `toml` crate, overlay `OSMPRJ_*` env vars — is straightforward and mirrors what the Python `Settings` class does.

**Alternatives considered:**
- *figment*: excellent layered config, but adds a dependency; revisit when Settings grows
- *config-rs*: similar tradeoff

### Toolchain management: pixi `rust` feature

Adding `rust` and `cargo` from conda-forge to a `rust` pixi feature means contributors run `pixi install` once and get both Python and Rust toolchains. No external `rustup` required.

## Risks / Trade-offs

- [tokio dependency is heavy for a CLI tool] → At placeholder stage the binary will be larger than necessary. Acceptable — it's the right foundation for async DB work.
- [conda-forge rust package may lag behind latest stable] → The scaffold only needs a recent stable; check version before pinning.
- [Two CLIs with the same command surface can diverge] → Intentional at this stage. Parity enforcement is a future concern once both implementations have real logic.
