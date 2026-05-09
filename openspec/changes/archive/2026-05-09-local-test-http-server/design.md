## Context

The e2e test suite in `tests/integration/test_sync_e2e.py` drives the osmprj binary as a
subprocess and observes its output. The binary downloads `.osm.pbf` files from
`download.geofabrik.de` and imports them via `osm2pgsql`. There is currently no way to intercept
those HTTP requests from the test layer.

The Geofabrik index (`index-v1.json`) is already intercepted via a fake cache: the conftest writes
a static JSON file to a tmp dir and sets `XDG_CACHE_HOME` to that dir. The binary reads the index
from that cache. However the fast tests use the `run` fixture (which injects the cache env var)
while the e2e tests use `run_cmd` (which does not) — so the e2e tests bypass the mock index and
hit the real Geofabrik server.

The binary also has a bug: `download_pbf()` creates the destination file and starts streaming into
it, but if the stream fails mid-way the partial file is left on disk. The lock file is not written
(error path), so a subsequent sync will re-download. The partial file is eventually overwritten,
but it means a failed sync leaves unexpected state.

## Goals / Non-Goals

**Goals:**

- All `download.geofabrik.de` HTTP requests are replaced with requests to a local server
- The local server can inject specific HTTP errors on a per-path, per-count basis
- PBF fixture files are cached in the repository tree (gitignored) so tests never re-download
- `download_pbf()` deletes the partial file before returning an error
- Five new error-scenario tests cover: 503, 429, 404, wrong MD5, partial cleanup

**Non-Goals:**

- Mocking `osm2pgsql-replication`'s replication server (`planet.openstreetmap.org`) — out of scope
- Adding retry logic to the binary for 503/429 — error tests verify fail-fast behavior only
- New Rust config fields — no `download_base_url` or similar setting is added

## Decisions

### URL rewriting: Python conftest, not Rust config

**Decision:** Rewrite URLs in a conftest fixture rather than adding a `download_base_url` field to
`ProjectSettings`.

**Rationale:** The Geofabrik index is already a replaceable file in the cache; the conftest
controls its contents. Adding a Rust config field would require writing the URL into every test's
`osmprj.toml`, coupling test data to an internal config knob. The Python approach is transparent
to the binary.

**Alternative considered:** An env-var override (`OSMPRJ_DOWNLOAD_BASE_URL`) read by the binary.
Rejected: adds production complexity for a testing concern.

### Server implementation: Python stdlib `ThreadingHTTPServer`

**Decision:** Use Python's built-in `http.server.ThreadingHTTPServer` with a custom handler.

**Rationale:** No new Python dependencies. `ThreadingHTTPServer` handles concurrent HEAD + GET
requests (the binary issues HEAD preflight then GET). The handler is simple enough to write
directly.

**Alternative considered:** `pytest-httpserver`. Rejected: adds a dependency; stdlib is sufficient.

### PBF fixture cache location

**Decision:** Default to `<repo_root>/tests/fixtures/pbf/`, overridable via `OSMPRJ_TEST_FIXTURE_DIR`.

**Rationale:** In-repo location keeps everything self-contained. The directory is gitignored.
The env var allows CI pipelines to point at a pre-populated artifact cache directory, avoiding a
network call even on first setup.

### Platform-aware cache env var as unified isolation boundary

**Decision:** Control the platform cache root via the correct OS env var for all server-based
subprocesses, mirroring the pattern already used by the existing `geofabrik_cache_dir` and `run`
fixtures:

| Platform | Env var set      | `dirs::cache_dir()` resolves to | Index written at                              |
|----------|------------------|---------------------------------|-----------------------------------------------|
| Linux    | `XDG_CACHE_HOME` | `<root>`                        | `<root>/osmprj/geofabrik-index-v1.json`       |
| macOS    | `HOME`           | `<root>/Library/Caches`         | `<root>/Library/Caches/osmprj/geofabrik-index-v1.json` |
| Windows  | `LOCALAPPDATA`   | `<root>`                        | `<root>/osmprj/geofabrik-index-v1.json`       |

A shared `_platform_cache_subdir(root)` helper encapsulates this logic: returns
`root / "Library" / "Caches"` on macOS, `root` otherwise.

**Rationale:** The binary's `effective_data_dir()` falls back to
`dirs::cache_dir()/osmprj/geofabrik/` when `data_dir` is not configured. By controlling the
platform cache root, both the Geofabrik index AND the downloaded PBF storage land inside a single
tmp tree — no writes to the real user cache, no shared state between test sessions.
All three platforms are exercised with no conditional logic in individual tests.

### Partial cleanup: wrap stream in inner async block

**Decision:** In `download_pbf()`, run the streaming loop inside an inner `async { ... }.await`
block. On `Err`, drop the file handle and call `tfs::remove_file(dest)` before returning.

**Rationale:** Minimal change, no restructuring of the outer function. The `drop(file)` is needed
before `remove_file` on Windows (which forbids deleting open files). `remove_file` failure is
silently ignored since the file may not exist and we're already in an error path.

**Alternative considered:** RAII guard (Drop impl that deletes the file). Rejected: more code for
the same outcome; the async block approach is idiomatic for this case.

### Error tests: `slow`, no DB required

**Decision:** Mark error tests `@pytest.mark.slow` but NOT `@pytest.mark.integration`.

**Rationale:** The binary checks for `osm2pgsql` on `PATH` before downloading, so the pixi dev
environment is required (hence `slow`). But error scenarios fail during download before import, so
no real DB is needed. Using a dummy `--db` URL is fine; the binary treats a failed DB connection
as "all sources fresh" and proceeds to download.

## Risks / Trade-offs

- **First-run network call:** `pbf_fixture_cache` still downloads Monaco PBF from Geofabrik on
  first use. After that it is never re-downloaded. CI must cache `tests/fixtures/pbf/` or set
  `OSMPRJ_TEST_FIXTURE_DIR` to a pre-populated artifact.
  → Mitigation: document the cache key in CI config; first-run is a one-time cost.

- **Port collision:** `ThreadingHTTPServer(("127.0.0.1", 0))` binds a random OS-assigned port;
  no collision risk. The port is read back from `server.server_address[1]` after bind.

- **Error injection state shared across session:** `download_server` is session-scoped. Injected
  errors must be reset after each error test to avoid bleeding into subsequent tests.
  → Mitigation: the `reset_server_errors` fixture calls `server.reset_errors()` in teardown;
  all error tests must declare it as a dependency.

- **Cross-platform cache resolution:** `data_dir` for the partial cleanup assertion must be
  computed via `_platform_cache_subdir(root) / "osmprj" / "geofabrik"` to correctly account for
  macOS's `Library/Caches` layer. Tests must not hardcode Linux paths.
  → Mitigation: `_platform_cache_subdir()` helper is shared between `geofabrik_cache_with_server`
  and the partial cleanup test; the pattern is identical to the existing `geofabrik_cache_dir` fixture.
