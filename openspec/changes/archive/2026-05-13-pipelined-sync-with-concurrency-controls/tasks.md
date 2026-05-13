## 1. Config — new concurrency fields

- [x] 1.1 Add `max_concurrent_downloads: Option<usize>` and `max_concurrent_imports: Option<usize>` to `ProjectSettings` in `src/config.rs`
- [x] 1.2 Add `effective_max_concurrent_downloads() -> usize` (default 3) and `effective_max_concurrent_imports() -> usize` (default 1) methods to `ProjectSettings`

## 2. Tuner — concurrency-aware RAM budget

- [x] 2.1 Add `concurrent_imports: usize` field to `TunerInput` in `src/tuner.rs`
- [x] 2.2 Update `get_cache_mb` to use `system_ram_gb / concurrent_imports as f64` as the RAM budget instead of `system_ram_gb`
- [x] 2.3 Update all `TunerInput` construction sites in `src/commands/sync.rs` to pass `concurrent_imports`
- [x] 2.4 Verify all existing tuner tests still compile and pass after adding `concurrent_imports` to `TunerInput` (detailed test additions are in section 9)

## 3. Pre-flight — HEAD requests and largest-first sort

- [x] 3.1 Add `fetch_content_length(client: &reqwest::Client, url: &str) -> u64` async fn in `src/commands/sync.rs` that issues a HEAD request and returns `Content-Length` (0 on failure or missing header)
- [x] 3.2 Before seeding the download JoinSet, resolve all URLs and fire all HEAD requests concurrently, collecting `(name, url, size)` tuples
- [x] 3.3 Sort the collected tuples by size descending (largest first)

## 4. Download phase — semaphore-capped JoinSet

- [x] 4.1 Add `tokio::sync::Semaphore` import and create `Arc<Semaphore>` with `effective_max_concurrent_downloads()` permits before the download JoinSet
- [x] 4.2 Update `download_source` signature to accept `Arc<Semaphore>`; acquire permit at the start of the function body and hold it until MD5 verification completes
- [x] 4.3 Seed the download JoinSet by iterating the sorted `(name, url, size)` list from the pre-flight phase

## 5. Import task — extract `import_source` fn

- [x] 5.1 Extract the current per-source import loop body into a new `async fn import_source(imp_sem: Arc<Semaphore>, config: ..., name: ..., pbf_path: ..., mp: Arc<MultiProgress>, ...)` in `src/commands/sync.rs`
- [x] 5.2 Inside `import_source`, acquire the import semaphore permit at the start and hold it for the full pipeline (osm2pgsql → post-process → replication init)
- [x] 5.3 Move the replication init call into `import_source`, immediately after post-processing succeeds, removing the separate end-of-sync replication loop
- [x] 5.4 Update spinner message through sub-phases: "Importing {name}..." → "Post-processing {name}..." → "Initialising replication for {name}..." → finish with "✓ {name} imported"
- [x] 5.5 Change `import_source` to return `Result<String, (String, OsmprjError)>` (source name on success, (name, error) on failure) instead of using early `return Err`

## 6. Unified event loop — replace phases 1 and 3b

- [x] 6.1 Create `imp_set: JoinSet` alongside the existing `dl_set: JoinSet`
- [x] 6.2 Create `dl_errors: Vec<(String, OsmprjError)>` and `imp_errors: Vec<(String, OsmprjError)>` for error collection
- [x] 6.3 Implement the `tokio::select!` loop body: on `dl_set.join_next()` result — update lock file on success, spawn `import_source` into `imp_set`; on `dl_set` error — push to `dl_errors`
- [x] 6.4 Implement the `imp_set.join_next()` arm: push success to `imported`, push failure to `imp_errors`
- [x] 6.5 Add the `if dl_set.is_empty() && imp_set.is_empty() { break; }` guard and `if !set.is_empty()` select guards
- [x] 6.6 After the loop, print all `dl_errors` and `imp_errors` together and return an aggregate error if either vec is non-empty
- [x] 6.7 Remove the now-redundant sequential fresh-import `for` loop (old Phase 3b) and the separate replication init loop

## 7. Unified MultiProgress display

- [x] 7.1 Wrap `MultiProgress` in `Arc` and pass it through to both `download_source` and `import_source`
- [x] 7.2 Remove the `mp.clear()` call that currently fires after download completion; clear only after the unified loop exits
- [x] 7.3 Remove the "files ready — starting imports" transition `println!` (superseded by the live unified display)
- [x] 7.4 Verify that finished download bars and import spinners print their final message line correctly above the live panel

## 8. Unit tests — `src/config.rs`

- [x] 8.1 Add test: `effective_max_concurrent_downloads` returns 3 when field is absent
- [x] 8.2 Add test: `effective_max_concurrent_downloads` returns the configured value when set
- [x] 8.3 Add test: `effective_max_concurrent_imports` returns 1 when field is absent
- [x] 8.4 Add test: `effective_max_concurrent_imports` returns the configured value when set

## 9. Unit tests — `src/tuner.rs`

- [x] 9.1 Update existing `input()` helper to accept `concurrent_imports: usize` and pass it through to `TunerInput`; update all existing call sites to pass `1` so current tests are unchanged
- [x] 9.2 Add test: `get_cache_mb` with `concurrent_imports = 2` uses half the RAM budget compared to `concurrent_imports = 1`
- [x] 9.3 Add test: `get_cache_mb` with `concurrent_imports = 4` uses one-quarter the RAM budget
- [x] 9.4 Add test: cache is still capped by per-import RAM budget (not full system RAM) when `slim_cache` exceeds the divided budget

## 10. Unit tests — `src/commands/sync.rs`

- [x] 10.1 Add test: `pbf_filename` produces correct output for a plain name (e.g. `"albania"` → `"albania.osm.pbf"`)
- [x] 10.2 Add test: `pbf_filename` replaces `/` with `-` for hierarchical names (e.g. `"north-america/us/alabama"` → `"north-america-us-alabama.osm.pbf"`)
- [x] 10.3 Add test: sorting logic — given a `Vec<(name, url, size)>`, verify that after sorting by size descending the order is largest-first (test the sort expression in isolation, extracted into a helper fn if needed)
- [x] 10.4 Add test: sort is stable for equal sizes — sources with the same `Content-Length` preserve their relative order

## 11. Verification

- [x] 11.1 Run `cargo test` and confirm all tests pass
- [x] 11.2 Run `cargo clippy` and resolve any warnings introduced by the new code
- [x] 11.3 Run `cargo build --release` and confirm a clean build
