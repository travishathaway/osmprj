## Why

`osmprj sync` is currently a stub. Without it, users must manually download OSM extracts, run osm2pgsql with hand-tuned flags, and wire up replication themselves — the entire reason the tool exists. Implementing sync delivers the core value proposition: one command that downloads, imports, and prepares a database for continuous updates.

## What Changes

- Implement `osmprj sync [source...]` — optionally filters to named sources; defaults to all sources in `osmprj.toml`
- Add `-v` / `--verbose` global flag to `Cli` (clap `global = true`) that streams osm2pgsql output live
- **Phase 1 — Downloads** (concurrent via `tokio::JoinSet`):
  - Geofabrik sources: stream-download `.osm.pbf` to `data_dir`, verify against `.osm.pbf.md5`, write progress bar per file (indicatif)
  - Local `--path` sources: skip download entirely
  - If any download fails: finish remaining downloads, report all failures, exit non-zero without importing
  - Write / update `osmprj.lock` after each successful download
- **Phase 2 — Imports** (sequential):
  - Clear download bars; print colorful "downloads complete" transition message with 🗺 emoji
  - Per source: build a tuned `osm2pgsql` invocation (ported from osm2pgsql-tuner), generate a lua style tempfile via cached osm2pgsql-themepark, spawn the process, show globe-spinning progress (🌍🌎🌏🌐), tee output to `log_dir/<source>.log`
  - Stop on first import failure
- **Post-import**: run `osm2pgsql-replication init -d <db_url> --schema <schema>` once per successfully imported source
- Add `project.data_dir`, `project.log_dir`, `project.ssd` to `ProjectSettings` and `osmprj.toml`
- Introduce `osmprj.lock` — TOML file written to project root, tracking per-source download metadata and themepark cache state
- Download and cache `osm2pgsql-themepark` from GitHub as a tarball on first sync; record in lock file

## Capabilities

### New Capabilities

- `sync-command`: Full `osmprj sync` implementation — download phase, import phase, replication init
- `osm2pgsql-tuner`: Ported tuning logic that computes optimal `--cache`, `--slim`, `--flat-nodes` flags from system RAM and PBF file size
- `themepark-cache`: Download, extract, and cache the osm2pgsql-themepark GitHub repo; generate per-source lua style configs
- `lock-file`: `osmprj.lock` data model and read/write logic

### Modified Capabilities

- `project-config`: `ProjectSettings` gains `data_dir`, `log_dir`, and `ssd` fields

## Impact

- `src/commands/sync.rs` — new file, bulk of the implementation
- `src/config.rs` — three new optional fields on `ProjectSettings`
- `src/main.rs` — add `verbose: bool` global flag; wire `sync` subcommand arguments
- `src/lock.rs` — new file for lock model and I/O
- `src/tuner.rs` — new file for osm2pgsql command builder
- `src/themepark.rs` — new file for themepark cache management and lua generation
- `Cargo.toml` — new dependencies: `sysinfo`, `futures`, `md-5`, `chrono`, `tempfile`, `flate2`, `tar`
- New test fixtures and integration tests in `tests/integration/`
