## Why

The Rust binary currently has five placeholder commands that print "not yet implemented." The two most foundational commands — `init` (which creates the project config file) and `add` (which registers OSM data sources) — need real implementations before any other command can work, since all subsequent commands depend on a valid `osmprj.toml`.

## What Changes

- Expand `src/config.rs` to represent the full `osmprj.toml` schema: top-level `database_url` plus `[sources.<name>]` sections with `theme`, `schema`, `path`, and `[sources.<name>.topics]` sub-tables
- Add `src/geofabrik.rs`: download and cache the Geofabrik `index-v1.json` once to the OS user cache dir; expose a lookup function that resolves a source name to a PBF download URL
- Implement `src/commands/init.rs`: create `osmprj.toml` in CWD with `database_url` from `--db` flag; error if file already exists
- Implement `src/commands/add.rs`: validate a Geofabrik ID against the cached index, then append a `[sources.<name>]` block to `osmprj.toml`; support `--path`/`--name` variant for local files
- Add new Cargo dependencies: `reqwest` (HTTP), `dirs` (OS cache path), `toml_edit` (non-destructive TOML mutation)
- Refactor `src/main.rs` `Add` and `Init` variants to accept arguments via clap

## Capabilities

### New Capabilities

- `project-config`: Full `osmprj.toml` data model — top-level settings, per-source config, topic customization, schema name normalization
- `geofabrik-index`: Geofabrik index-v1.json download, persistent cache, and ID→URL lookup
- `rust-init-command`: `osmprj init [--db <url>]` — creates `osmprj.toml` in CWD
- `rust-add-command`: `osmprj add <geofabrik-id> [--theme <theme>] [--schema <schema>]` and `osmprj add --path <file> --name <label> [--theme <theme>]`

### Modified Capabilities

## Impact

- `src/config.rs` is replaced with a richer model — the existing `Settings::load()` becomes the basis for the new `ProjectConfig::load()`
- `src/main.rs` `Init` and `Add` arms gain arguments; other arms are unchanged placeholders
- New source files: `src/geofabrik.rs`, `src/commands/init.rs`, `src/commands/add.rs`
- New Cargo dependencies: `reqwest` (with `blocking` or `rustls-tls`), `dirs`, `toml_edit`
- No changes to the Python package
