## Context

The Rust binary has a working `init` and `add` but `sync` is a stub. The command must orchestrate several external systems: Geofabrik's HTTP download servers, the local filesystem, the `osm2pgsql` binary, and the `osm2pgsql-replication` Python script. The existing codebase already has `tokio` (full features), `reqwest` (blocking + rustls), and `indicatif` in `Cargo.toml` — only targeted additions are needed.

## Goals / Non-Goals

**Goals:**
- Concurrent async downloads with per-file progress bars and MD5 verification
- Tuned `osm2pgsql` invocation derived from system RAM + PBF size (slim/flat-nodes/cache)
- Lua style config via cached osm2pgsql-themepark; tempfile only when topic customization exists
- Per-source log files; live streaming with `-v`
- `osmprj.lock` written incrementally (after each download)
- `osm2pgsql-replication init` per source after all imports succeed

**Non-Goals:**
- `osm2pgsql-replication update` (subsequent incremental updates — a future command)
- Windows or macOS support (Linux-only for now; `/proc/meminfo` RAM detection)
- Parallel imports (sequential only; concurrent DB writes would cause contention)
- Resumable downloads
- Authentication for Geofabrik subscribers (internal PBF URLs)

## Decisions

### D1: reqwest blocking → async for downloads

The geofabrik index fetch stays blocking (fast, < 1 MB). Downloads switch to async `reqwest::Client` (non-blocking) so multiple downloads run concurrently inside the existing `tokio::main` runtime. Both modes coexist in the same binary — no feature flag change needed beyond removing the `blocking` requirement for downloads.

### D2: `tokio::JoinSet` over `futures::join_all`

`JoinSet` allows tasks to be spawned dynamically and results collected as they complete. This enables incremental lock-file writes (after each task finishes) rather than waiting for all downloads. Errors are collected into a `Vec` and reported together; the set drains fully before returning.

### D3: Lock written after each successful download, not at end

Writing the lock incrementally means a partial sync (interrupted or partially failed) still records what did complete. The lock is a `HashMap<SourceName, LockEntry>` loaded at startup and flushed to TOML after each successful download using `toml_edit` (preserves formatting/comments).

### D4: Always `--slim`, never `--drop`

Since `osm2pgsql-replication` requires middle tables to survive, every import uses `--slim` without `--drop`. The tuner's `run_in_ram` path (which skips `--slim`) is never taken. This simplifies the ported tuner: `slim_no_drop = true` and `append_first_run = true` are constants, so only the `--cache`, `--flat-nodes`, and `--schema` decisions remain dynamic.

Tuner logic ported to `src/tuner.rs`:
```
cache_max_gb   = system_ram_gb * 0.66
noslim_cache   = 1.0 + (2.5 * pbf_gb)
slim_cache     = 0.75 * noslim_cache

flat_nodes = (pbf_gb >= 8.0 && ssd) || pbf_gb >= 30.0
cache_mb   = if flat_nodes { 0 }
             else if slim_cache > cache_max_gb { (cache_max_gb * 1024) as u32 }
             else { (slim_cache * 1024) as u32 }
```

Command template:
```
osm2pgsql --create --slim
  [--cache=N]
  [--flat-nodes=<data_dir>/<source>.nodes]
  --output=flex --style=<lua_path>
  --database=<database_url>
  --schema=<effective_schema>
  <pbf_path>
```

### D5: System RAM via `/proc/meminfo`

`sysinfo` crate reads `/proc/meminfo` cross-platform. Provides `System::new_with_specifics` + `total_memory()` in bytes. One new dependency; cleaner than shelling out to `free`.

### D6: Themepark as a cached GitHub tarball

Download `https://github.com/osm2pgsql-dev/osm2pgsql-themepark/archive/refs/heads/master.tar.gz` to `<cache_dir>/osmprj/themepark/` on first sync if absent. Extract with `flate2` + `tar`. Record `cached_at` timestamp in `osmprj.lock`. Future: replaced by conda package detection.

Lua path resolution: generated tempfiles set `package.path` to the cached `lua/` directory:
```lua
package.path = '/home/user/.cache/osmprj/themepark/osm2pgsql-themepark-master/lua/?.lua;' .. package.path
local themepark = require('themepark')
```

### D7: Lua tempfile only when topics are customized

If a source has no `[topics]` block: pass the cached themepark config file directly as `--style` (e.g., `themepark/…/config/shortbread.lua`). The theme name maps to config filename: `shortbread_v1` → `shortbread.lua`, `shortbread_v1_gen` → `shortbread_gen.lua`, `basic` → `generic.lua` (with a fallback table).

If topics are customized: generate a `NamedTempFile`, write lua with `package.path` header + `add_topic` calls derived by reading the base config and applying `topics.add` / `topics.remove` / `topics.list`. The tempfile is held alive for the duration of the `osm2pgsql` subprocess via a `_guard` binding, then dropped (deleted) automatically.

### D8: Globe spinner via indicatif tick strings

```rust
ProgressStyle::with_template("{spinner} {msg}")
    .tick_strings(&["🌍 ", "🌎 ", "🌏 ", "🌐 ", "🌍 "])
```
Tick interval: 250 ms. Message format: `"Importing <source>..."`.

Download bars use `{spinner} {msg} [{bar:40}] {bytes}/{total_bytes} ({bytes_per_sec}, eta {eta})`.

During the download→import transition, all download bars are finished with `finish_and_clear()` and a single summary line is printed: `🗺  All downloads complete — starting imports`.

### D9: osm2pgsql-replication init per source

```
osm2pgsql-replication init -d <database_url> --schema <effective_schema>
```

Run after all imports succeed, once per source. This initialises the `osm2pgsql_properties` table in the source's schema with the replication server URL derived from the PBF download URL. Each source maps to its own schema, so replication state is isolated per source.

### D10: Import failure stops immediately

If `osm2pgsql` exits non-zero, log the stderr, print an error message, and return without processing remaining sources or running replication init. The lock file retains all successful download entries (written incrementally).

### D11: Log file tee

`tokio::process::Command` with `stdout(Stdio::piped())` and `stderr(Stdio::piped())`. Two async tasks read from each pipe, writing to `BufWriter<File>` and optionally to a `println!`/`eprintln!` if `--verbose`. The spinner is stopped before any verbose output is flushed to avoid interleaving.

## Risks / Trade-offs

- **Themepark tarball drift** — downloading `master` means the cached version may diverge from what tested configs expect. Mitigation: record `cached_at` in lock; add `osmprj themepark update` command later.
- **Geofabrik rate limits** — concurrent downloads from the same IP may be throttled. Mitigation: no retry logic in v1; users re-run if throttled. A `--concurrency N` flag is easy to add later.
- **`/proc/meminfo` Linux-only** — `sysinfo` has a cross-platform API so this is a library concern, not ours; however the build will only be tested on Linux-64.
- **Large flat-nodes files** — a `europe.nodes` file can be 50+ GB. Storing in `data_dir` is correct, but the user must have sufficient space. No space pre-check in v1.
- **osm2pgsql-replication requires Python** — `osm2pgsql-replication` is a Python script shipped with the `osm2pgsql` package or separately. In the pixi environment it's available; outside pixi it may not be. Mitigation: check `which osm2pgsql-replication` at startup and emit a clear error if absent.

## Open Questions

- Should `osmprj sync` print a warning (not error) if `database_url` is not set in config, and skip the import phase entirely (download-only mode)? Useful for pre-caching data.
- Theme-name → config-filename mapping: maintain a hardcoded table in `src/themepark.rs` or derive from directory listing of the cached `config/` folder? Directory listing is more forward-compatible.
