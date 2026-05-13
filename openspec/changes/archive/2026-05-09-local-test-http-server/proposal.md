## Why

The e2e test suite downloads real `.osm.pbf` files from `download.geofabrik.de` on every run,
creating a network dependency, burdening a public server, and making it impossible to test error
scenarios (HTTP 503, 429, 404, MD5 mismatch, partial downloads). Additionally, partial downloads
are silently left as corrupt files on disk when a download fails mid-stream.

## What Changes

- Add a local test HTTP server (Python `ThreadingHTTPServer`) that serves cached PBF fixtures
  and supports injecting HTTP error conditions per-request
- Cache the Monaco `.osm.pbf` fixture in `tests/fixtures/pbf/` (gitignored, configurable via
  `OSMPRJ_TEST_FIXTURE_DIR`) — downloaded once from Geofabrik, then never again
- Rewrite the cached Geofabrik index at fixture time to point URLs at `localhost`, so the binary
  downloads from the local server without any Rust config changes
- Replace all existing e2e tests that hit `download.geofabrik.de` with equivalents using the
  local server
- Add five new error-scenario tests: HTTP 503, 429, 404, wrong MD5, and partial download cleanup
- **Fix bug:** `download_pbf()` in `sync.rs` does not delete the destination file when streaming
  fails mid-way; fix it to clean up the partial file before propagating the error

## Capabilities

### New Capabilities

- `test-http-server`: Local test HTTP server fixture with error injection for e2e tests

### Modified Capabilities

- `sync-command`: `download_pbf()` must delete the partial file on stream failure

## Impact

- `src/commands/sync.rs` — one-function change to `download_pbf()`
- `tests/integration/conftest.py` — new fixtures (no new Python dependencies)
- `tests/integration/test_sync_e2e.py` — refactored + 5 new tests
- `tests/fixtures/` — new gitignored directory for PBF fixture cache
- `.gitignore` — one new entry
