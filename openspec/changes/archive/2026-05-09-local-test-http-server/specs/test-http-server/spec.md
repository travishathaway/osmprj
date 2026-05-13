## ADDED Requirements

### Requirement: PBF fixture cache
The test suite SHALL maintain a local cache of `.osm.pbf` fixtures so that no network requests to
`download.geofabrik.de` are made during test runs after the first setup. The cache location SHALL
default to `<repo_root>/tests/fixtures/pbf/` and SHALL be overridable via the
`OSMPRJ_TEST_FIXTURE_DIR` environment variable. The cache directory SHALL be listed in `.gitignore`.
On first use the fixture SHALL be downloaded from `download.geofabrik.de` and a `.md5` sidecar
SHALL be computed and stored alongside it.

#### Scenario: Cache populated on first use
- **WHEN** `OSMPRJ_TEST_FIXTURE_DIR` is unset and `tests/fixtures/pbf/europe/monaco-latest.osm.pbf` does not exist
- **THEN** the `pbf_fixture_cache` fixture downloads the file from Geofabrik and saves it to that path

#### Scenario: Cache reused on subsequent runs
- **WHEN** `tests/fixtures/pbf/europe/monaco-latest.osm.pbf` already exists
- **THEN** no download is attempted and the existing file is used

#### Scenario: Custom cache location via env var
- **WHEN** `OSMPRJ_TEST_FIXTURE_DIR=/ci/pbf-cache` is set and the file exists at that path
- **THEN** the fixture serves the file from `/ci/pbf-cache/europe/monaco-latest.osm.pbf`

### Requirement: Local test HTTP server
The test suite SHALL provide a `download_server` session-scoped pytest fixture that starts a
`ThreadingHTTPServer` on `127.0.0.1` with an OS-assigned port. The server SHALL handle `GET` and
`HEAD` requests for `.osm.pbf` and `.osm.pbf.md5` paths by serving the corresponding files from
the PBF fixture cache. The server SHALL be stopped automatically on session teardown.

#### Scenario: Serves PBF file on GET
- **WHEN** the binary sends `GET /europe/monaco-latest.osm.pbf`
- **THEN** the server responds 200 with the cached file body

#### Scenario: Returns Content-Length on HEAD
- **WHEN** the binary sends `HEAD /europe/monaco-latest.osm.pbf`
- **THEN** the server responds 200 with a `Content-Length` header matching the file size

#### Scenario: Serves MD5 sidecar on GET
- **WHEN** the binary sends `GET /europe/monaco-latest.osm.pbf.md5`
- **THEN** the server responds 200 with `<md5hex>  monaco-latest.osm.pbf\n`

### Requirement: Error injection
The `TestOsmServer` class SHALL support injecting HTTP error responses for specific URL paths on a
per-count basis. Injected errors SHALL be consumed in order and SHALL NOT affect requests after the
count is exhausted. The server SHALL provide a `reset_errors()` method that clears all pending
injections. All injection state SHALL be protected by a threading lock.

#### Scenario: Status code injection consumed once
- **WHEN** `server.inject_error("/europe/monaco-latest.osm.pbf", status=503, times=1)` is called
  and the binary makes one GET request to that path
- **THEN** the first request receives HTTP 503 and the second request receives HTTP 200

#### Scenario: Bad MD5 injection
- **WHEN** `server.inject_bad_md5("/europe/monaco-latest.osm.pbf.md5", times=1)` is called
- **THEN** the next MD5 sidecar request for that path receives a response with an incorrect hash

#### Scenario: Partial response injection
- **WHEN** `server.inject_partial("/europe/monaco-latest.osm.pbf", bytes_to_send=1024, times=1)` is called
- **THEN** the server sends a 200 response with the real `Content-Length` but closes the connection after 1024 bytes, causing the client to observe a truncated body

#### Scenario: Reset clears all injections
- **WHEN** `server.reset_errors()` is called
- **THEN** all pending injections are removed and subsequent requests receive normal responses

### Requirement: Geofabrik index URL rewriting
The `geofabrik_cache_with_server` fixture SHALL create a modified copy of
`tests/data/geofabrik-index-v1.json` in which every `properties.urls.pbf` value has its
`https://download.geofabrik.de` prefix replaced with the test server's `base_url`. The modified
index SHALL be written to a tmp directory following the platform-correct cache layout (matching the
existing `geofabrik_cache_dir` fixture's conventions) so that the binary reads it via the
appropriate platform cache env var. Setting this env var SHALL also redirect `effective_data_dir()`
so all downloaded PBFs land in the same isolated tmp tree with no writes to the real user cache.

The index file SHALL be placed at the path that `dirs::cache_dir()` resolves to on each platform:
- Linux: `<root>/osmprj/geofabrik-index-v1.json` (set `XDG_CACHE_HOME=<root>`)
- macOS: `<root>/Library/Caches/osmprj/geofabrik-index-v1.json` (set `HOME=<root>`)
- Windows: `<root>/osmprj/geofabrik-index-v1.json` (set `LOCALAPPDATA=<root>`)

#### Scenario: PBF URL points to localhost
- **WHEN** the binary resolves the download URL for `monaco` using the injected index
- **THEN** the URL is `http://127.0.0.1:<port>/europe/monaco-latest.osm.pbf`

#### Scenario: No writes to the real user cache during tests
- **WHEN** any slow e2e test runs with the server fixtures on any platform
- **THEN** no files are written to the real platform cache directory (e.g. `~/.cache` on Linux,
  `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows)

### Requirement: Server-aware run_cmd fixture
The `run_cmd_with_server` session-scoped fixture SHALL behave identically to `run_cmd` but SHALL
inject the platform-appropriate cache env var (set to `geofabrik_cache_with_server`) into the
subprocess environment so that all invocations of the binary use the rewritten index and the
isolated data dir. The env var SHALL be selected by platform: `XDG_CACHE_HOME` on Linux,
`HOME` on macOS, `LOCALAPPDATA` on Windows.

#### Scenario: Platform cache env var injected into subprocess on Linux
- **WHEN** `run_cmd_with_server("sync", cwd=project)` is called on Linux
- **THEN** the binary subprocess receives `XDG_CACHE_HOME` pointing to the fixture tmp dir

#### Scenario: Platform cache env var injected into subprocess on macOS
- **WHEN** `run_cmd_with_server("sync", cwd=project)` is called on macOS
- **THEN** the binary subprocess receives `HOME` pointing to the fixture tmp dir

#### Scenario: Platform cache env var injected into subprocess on Windows
- **WHEN** `run_cmd_with_server("sync", cwd=project)` is called on Windows
- **THEN** the binary subprocess receives `LOCALAPPDATA` pointing to the fixture tmp dir

### Requirement: Per-test error state cleanup
The `reset_server_errors` function-scoped fixture SHALL call `download_server.reset_errors()` in
its teardown phase. Tests that inject errors MUST declare `reset_server_errors` as a dependency to
prevent error state from leaking into subsequent tests.

#### Scenario: Injected error does not bleed to next test
- **WHEN** a test injects a 503 error and finishes (with `reset_server_errors` declared)
- **THEN** the next test that requests the same path receives a normal 200 response

### Requirement: Error scenario test coverage
The test suite SHALL include the following error scenario tests, all marked `@pytest.mark.slow`
and NOT `@pytest.mark.integration`. Each test SHALL run `osmprj sync` with a dummy DB URL (DB
connection failure is gracefully handled as "all sources fresh"), inject the error via the server,
and assert both a non-zero exit code and a meaningful error message in stderr.

#### Scenario: HTTP 503 produces non-zero exit and error message
- **WHEN** the server returns 503 for the PBF download
- **THEN** `osmprj sync` exits non-zero and stderr contains "503" or "download"

#### Scenario: HTTP 429 produces non-zero exit and error message
- **WHEN** the server returns 429 for the PBF download
- **THEN** `osmprj sync` exits non-zero and stderr contains "429" or "download"

#### Scenario: HTTP 404 produces non-zero exit and error message
- **WHEN** the server returns 404 for the PBF download
- **THEN** `osmprj sync` exits non-zero and stderr contains "404"

#### Scenario: Wrong MD5 produces non-zero exit and error message
- **WHEN** the server returns an incorrect MD5 sidecar
- **THEN** `osmprj sync` exits non-zero and stderr contains "md5" or "mismatch" or "checksum" (case-insensitive)

#### Scenario: Partial download produces non-zero exit and no residual file
- **WHEN** the server closes the connection after 1024 bytes
- **THEN** `osmprj sync` exits non-zero and no `.osm.pbf` file exists in the data dir
