## Context

`osmprj sync` currently runs in two strictly sequential phases: (1) all downloads complete via an unbounded `tokio::task::JoinSet`, then (2) all imports run one-at-a-time in a serial `for` loop. With 10+ large sources (>100 MB each), this leaves the database idle during the entire download phase and CPU/network idle during the entire import phase. The Geofabrik server supports HEAD requests with `Content-Length`, enabling pre-flight size probing. The existing tuner allocates RAM as if each import owns the full machine.

## Goals / Non-Goals

**Goals:**
- Cap concurrent downloads with a configurable semaphore (default 3)
- Cap concurrent imports with a configurable semaphore (default 1, safe for large files)
- Pipeline downloads and imports so each import begins as soon as its download finishes
- Pre-flight HEAD requests determine file sizes; sources sorted largest-first before downloading
- Divide tuner RAM budget by `max_concurrent_imports` to prevent memory over-subscription
- Collect all errors across both phases and report at the end (no abort-on-first-failure)
- Fold replication init into the per-source import task
- Unified `MultiProgress` panel showing download bars and import spinners simultaneously

**Non-Goals:**
- Parallelising the update (replication) phase — only the fresh-import path changes
- Adaptive or auto-tuned concurrency limits — user configures them explicitly
- Changing the lock file format, `DownloadResult`, `run_subprocess`, or `pipe_to_log`
- Breaking changes to `osmprj.toml` (all new fields are optional with defaults)

## Decisions

### D1: Semaphore-per-phase, not a global work queue

**Decision:** Use two independent `Arc<tokio::sync::Semaphore>` instances — one for downloads, one for imports.

**Rationale:** Downloads and imports are resource-distinct (network vs. CPU/RAM/DB). A single global semaphore would force a fixed ratio between them and prevent the natural overlap where N downloads run while M imports run. Independent semaphores allow each phase to saturate its own resource independently.

**Alternative considered:** A priority work queue where tasks are re-prioritised as downloads complete. Rejected: significant complexity, no meaningful benefit over two semaphores for this use case.

### D2: Unified `tokio::select!` loop over two JoinSets

**Decision:** Replace the current sequential phases 1 and 3b with a single event loop that `select!`s over `dl_set: JoinSet` and `imp_set: JoinSet`.

```
loop {
    if dl_set.is_empty() && imp_set.is_empty() { break; }
    tokio::select! {
        Some(r) = dl_set.join_next(), if !dl_set.is_empty() => {
            // on download complete: update lock, spawn import task
        }
        Some(r) = imp_set.join_next(), if !imp_set.is_empty() => {
            // on import complete: record success or push error
        }
    }
}
```

**Rationale:** `select!` with `JoinSet::join_next()` is the idiomatic Tokio pattern for waiting on heterogeneous concurrent task pools. The `if !set.is_empty()` guards prevent selecting on an empty set returning `None` spuriously.

**Alternative considered:** A single `JoinSet` with a tagged enum result. Rejected: muddies the two phases and makes the semaphore scoping harder to reason about.

### D3: Semaphore permit held for the full task duration

**Decision:** Each `download_source` task acquires the download permit before the HTTP request and holds it until the MD5 verify completes. Each `import_source` task acquires the import permit before `osm2pgsql` and holds it until replication init completes.

**Rationale:** The permit represents active resource consumption. For downloads, the connection and disk I/O are the resource. For imports, RAM and DB connections are the resource — both are held through the full osm2pgsql → post-process → replication-init pipeline. Releasing early would allow a new import to start before the previous one has fully freed its resources.

### D4: Largest-first download ordering via HEAD pre-flight

**Decision:** Before seeding the `JoinSet`, issue one HEAD request per source to get `Content-Length`, sort sources by size descending, then spawn tasks in that order (all acquire semaphore internally).

**Rationale:** Largest-first ensures the longest downloads are farthest along when smaller ones finish, minimising the tail — the period where small imports are complete but large downloads are still running. All HEAD requests are fired concurrently (trivial overhead). If a HEAD request fails or returns no `Content-Length`, that source falls to the back of the sorted list (size treated as 0) rather than failing the whole sync.

**Alternative considered:** Sort smallest-first to start imports sooner. Rejected: with concurrent imports capped at M, early small imports may exhaust the import semaphore and block larger imports that need the same slot anyway. Largest-first produces better total pipeline throughput.

### D5: Per-import RAM budget = system_ram / max_concurrent_imports

**Decision:** Add `concurrent_imports: usize` to `TunerInput`. In `get_cache_mb`, replace `system_ram_gb * 0.66` with `(system_ram_gb / concurrent_imports as f64) * 0.66`.

**Rationale:** Each concurrent osm2pgsql process independently computes its cache size without awareness of siblings. With M=2 and 32 GB RAM, both would claim ~21 GB each (42 GB total), causing swapping. Dividing upfront gives each process a safe slice of the available RAM.

**Note:** `use_flat_nodes` thresholds are unchanged — flat-nodes is an I/O strategy, not a RAM strategy. A file that warrants flat-nodes still does regardless of concurrency.

### D6: Error collection instead of abort-on-first-failure

**Decision:** Replace `return Err(...)` on import failure with `imp_errors.push(...)`. After the event loop, all download errors and import errors are reported together, then the command returns an aggregate error.

**Rationale:** With concurrent imports, an abort-on-first-failure would leave sibling tasks orphaned mid-import. Collecting errors allows in-flight work to complete naturally, gives the user a full picture of what failed, and allows partial success (some sources imported, others not) to be reflected in the lock file.

### D7: Unified MultiProgress — downloads as bars, imports as mutating spinners

**Decision:** A single `Arc<MultiProgress>` is created at the start of the sync and passed into both download and import tasks. Download tasks add a `ProgressBar` (byte progress). Import tasks add a `ProgressBar` spinner whose message mutates through sub-phases: "Importing {name}..." → "Post-processing {name}..." → "Initialising replication for {name}..." → finish.

**Rationale:** Users with 10 sources benefit from seeing the full pipeline state at a glance. A single `MultiProgress` instance handles concurrent rendering correctly via `indicatif`'s internal draw loop. Finished bars print their final message and scroll out of the live region automatically.

## Risks / Trade-offs

**[Risk] Memory pressure with M>1 imports and large files**
→ Mitigation: Default `max_concurrent_imports = 1`. Division of RAM budget (D5) prevents over-allocation. Users who increase M do so explicitly with awareness.

**[Risk] Geofabrik HEAD requests add latency before first download starts**
→ Mitigation: All HEAD requests are fired concurrently. For 10 sources this is a handful of round-trips (~1-2s) — negligible against download times of minutes.

**[Risk] HEAD request fails or returns no Content-Length**
→ Mitigation: Treat missing size as 0, sort that source to the back. Do not fail sync. The download proceeds normally (progress bar shows indeterminate length, same as today when GET response lacks Content-Length).

**[Risk] Import concurrency with postgreSQL connection limits**
→ Mitigation: Each osm2pgsql process holds one DB connection. With M=2 that's 2 simultaneous connections — well within any default PostgreSQL limit. Document this in config comments.

**[Risk] `JoinSet` + `select!` loop complexity vs. current simple sequential code**
→ Mitigation: Extract into well-named functions (`download_source`, `import_source`). The loop body stays small. Add inline comments explaining the select! guard idiom.

## Migration Plan

No migration required. All new `osmprj.toml` fields are optional. Existing projects get the new defaults (3 concurrent downloads, 1 concurrent import) without any config changes. Behaviour for single-source projects is identical to today.
