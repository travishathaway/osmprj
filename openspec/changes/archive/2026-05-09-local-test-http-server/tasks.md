## 1. Rust: Partial download cleanup

- [x] 1.1 In `src/commands/sync.rs` `download_pbf()`, wrap the streaming while-loop in an inner `async { ... }.await` block assigned to `stream_result`
- [x] 1.2 After the inner block, add `if stream_result.is_err() { drop(file); let _ = tfs::remove_file(dest).await; }`
- [x] 1.3 Return `stream_result` at the end of `download_pbf()`
- [x] 1.4 Run `cargo fmt` and `cargo clippy` — address any warnings
- [x] 1.5 Run `cargo test` — verify existing unit tests pass

## 2. Repository: gitignore and fixture directory

- [x] 2.1 Add `tests/fixtures/` to `.gitignore`

## 3. Python: PBF fixture cache fixture

- [x] 3.1 In `tests/integration/conftest.py`, add `REPO_ROOT` and `PBF_FIXTURE_DIR` constants (default `<repo_root>/tests/fixtures/pbf/`, overridable via `OSMPRJ_TEST_FIXTURE_DIR`)
- [x] 3.2 Add session-scoped `pbf_fixture_cache` fixture that creates the cache dir, downloads `europe/monaco-latest.osm.pbf` from Geofabrik if absent, computes and saves the `.md5` sidecar (`<hash>  <filename>\n` format), and returns the cache `Path`

## 4. Python: TestOsmServer class

- [x] 4.1 Add `TestOsmServer` class to `conftest.py` with `__init__(fixture_dir, port=0)`, a `threading.Lock`-protected `_errors` dict, and `base_url` property
- [x] 4.2 Implement `inject_error(path, *, status, times=1)` method
- [x] 4.3 Implement `inject_bad_md5(path, *, times=1)` method
- [x] 4.4 Implement `inject_partial(path, *, bytes_to_send, times=1)` method
- [x] 4.5 Implement `reset_errors()` method
- [x] 4.6 Implement `start()` and `stop()` methods using `ThreadingHTTPServer` and a daemon thread
- [x] 4.7 Implement the custom `BaseHTTPRequestHandler` with `do_HEAD` and `do_GET`: resolve path to fixture file, check injection state (consume one count), serve file or injected response. For partial: send real `Content-Length`, write `bytes_to_send` bytes, then close connection

## 5. Python: Session fixtures for server integration

- [x] 5.1 Add session-scoped `download_server` fixture: instantiate `TestOsmServer(pbf_fixture_cache)`, start, yield, stop
- [x] 5.2 Add module-level `_platform_cache_subdir(root: Path) -> Path` helper: returns `root / "Library" / "Caches"` on macOS (`platform.system() == "Darwin"`), `root` on all other platforms
- [x] 5.3 Add module-level `_platform_cache_env(root: Path) -> tuple[str, str]` helper: returns `("XDG_CACHE_HOME", str(root))` on Linux, `("HOME", str(root))` on macOS, `("LOCALAPPDATA", str(root))` on Windows
- [x] 5.4 Add session-scoped `geofabrik_cache_with_server` fixture: create `root = tmp_path_factory.mktemp(...)`, compute `osmprj_dir = _platform_cache_subdir(root) / "osmprj"`, create it, load `tests/data/geofabrik-index-v1.json`, replace all `https://download.geofabrik.de` occurrences in `properties.urls.pbf` with `download_server.base_url`, write modified JSON to `osmprj_dir / "geofabrik-index-v1.json"`, return `root`
- [x] 5.5 Add session-scoped `run_cmd_with_server` fixture: copy `os.environ`, call `_platform_cache_env(geofabrik_cache_with_server)` and set the returned key/value in the env copy, pass env to `subprocess.run`, same `check=` kwarg API as `run_cmd`
- [x] 5.6 Add function-scoped `reset_server_errors` fixture (not autouse): yield, then call `download_server.reset_errors()`

## 6. Python: Refactor existing e2e tests

- [x] 6.1 In `test_sync_e2e.py`, update the `source_state` fixture to accept `run_cmd_with_server` instead of `run_cmd` (the cache env injection is now handled inside `run_cmd_with_server`)
- [x] 6.2 Verify existing `test_first_sync_imports_source`, `test_second_sync_runs_update`, and `test_replication_timestamp_advances` still pass using the local server

## 7. Python: New error scenario tests

- [x] 7.1 Add `test_sync_503_fails_with_clear_error`: init project with dummy DB URL, add monaco, inject 503 on PBF path, run sync, assert rc ≠ 0 and "503" or "download" in stderr (case-insensitive); declare `reset_server_errors`
- [x] 7.2 Add `test_sync_429_fails_with_clear_error`: same pattern with status 429
- [x] 7.3 Add `test_sync_404_fails_with_clear_error`: same pattern with status 404, assert "404" in stderr
- [x] 7.4 Add `test_sync_wrong_md5_fails_with_clear_error`: inject bad MD5 on `.md5` path, run sync, assert rc ≠ 0 and "md5" or "mismatch" or "checksum" in stderr (case-insensitive); declare `reset_server_errors`
- [x] 7.5 Add `test_sync_partial_download_cleans_up`: inject partial (1024 bytes) on PBF path, run sync, assert rc ≠ 0, resolve `data_dir` as `_platform_cache_subdir(geofabrik_cache_with_server) / "osmprj" / "geofabrik"`, assert no `.osm.pbf` files exist in that dir; declare `reset_server_errors`

## 8. Verification

- [x] 8.1 Run `pytest -m "not slow"` — all fast tests pass (no changes to fast-test fixtures)
- [x] 8.2 Run `pixi run --environment dev pytest -m slow -v` — all slow tests pass including the five new error tests
- [x] 8.3 Confirm no `.osm.pbf` files are created under the real platform cache directory (`~/.cache` on Linux, `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows) during the test run
