## Why

When a project has many sources (10+ regions, e.g. multiple U.S. states or European countries), the sync command downloads all files simultaneously with no cap, then imports them one-by-one only after every download has finished. This leaves CPU and I/O idle during the download phase and wastes the opportunity to start importing small regions while large ones are still downloading.

## What Changes

- Add `max_concurrent_downloads` config field (default: 3) to cap simultaneous downloads via a semaphore
- Add `max_concurrent_imports` config field (default: 1) to cap simultaneous osm2pgsql imports via a semaphore
- Introduce a pre-flight phase that issues HEAD requests to determine file sizes and sorts sources largest-first before downloading
- Merge the current sequential download and import phases into a unified pipelined loop: imports begin as soon as their download completes, without waiting for all downloads to finish
- Fold replication init into the per-source import task (runs immediately after post-processing, not in a separate final loop)
- Change import failure handling from abort-on-first-error to collect-all-errors-then-report
- Update osm2pgsql tuner to divide RAM budget by `max_concurrent_imports` so concurrent processes do not over-allocate memory
- Replace separate download progress bars and import spinners with a unified `MultiProgress` panel showing both simultaneously; import spinner message mutates through sub-phases (importing → post-processing → initialising replication)

## Capabilities

### New Capabilities

- `sync-concurrency-controls`: Configuration and enforcement of per-phase concurrency limits (download semaphore, import semaphore) and the pre-flight HEAD-request size-probe with largest-first sort order
- `pipelined-sync`: The unified download+import event loop that pipelines work across phases, collects errors without aborting, and folds replication init into the per-source import task

### Modified Capabilities

- `sync-command`: The sync execution model changes significantly — phases are no longer strictly sequential, error handling becomes non-fatal per source, and the progress display merges into a single live panel
- `osm2pgsql-tuner`: RAM budget calculation must be divided by the number of concurrent imports to avoid memory over-subscription

## Impact

- `src/config.rs`: Two new optional fields on `ProjectSettings`; two new `effective_*` methods
- `src/tuner.rs`: `TunerInput` gains `concurrent_imports: usize`; `get_cache_mb` uses per-import RAM budget
- `src/commands/sync.rs`: Phase 0 (HEAD pre-flight + sort), merged Phase 1+3b (unified `select!` loop with two `JoinSet`s and two semaphores), new `import_source` async fn, error collection refactor, `MultiProgress` passed into import tasks
- No changes to lock file format, `DownloadResult` struct, `run_subprocess`, or `pipe_to_log`
- No breaking changes to `osmprj.toml` schema (new fields are optional with defaults)
