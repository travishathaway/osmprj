## ADDED Requirements

### Requirement: All HTTP requests MUST be async
The system SHALL use only the async `reqwest` API (`reqwest::Client`, not `reqwest::blocking`) for all downloads and MD5 sidecar fetches. The `blocking` feature of the `reqwest` crate SHALL NOT be enabled in `Cargo.toml`. Mixing the blocking transport with a Tokio runtime causes a runtime-within-runtime panic on macOS.

#### Scenario: No blocking reqwest feature
- **WHEN** `Cargo.toml` is inspected
- **THEN** the `reqwest` dependency does not include `"blocking"` in its feature list

#### Scenario: PBF download uses async client
- **WHEN** a PBF file is streamed from Geofabrik
- **THEN** the download uses an `Arc<reqwest::Client>` and `.await`, never blocking calls

#### Scenario: MD5 sidecar fetch uses async client
- **WHEN** the `.md5` sidecar file is fetched after a download
- **THEN** it uses the same async client as the download, never blocking calls

### Requirement: Sync accepts optional source filter
`osmprj sync` SHALL accept zero or more positional `source` arguments. When provided, only those sources SHALL be processed. When omitted, all sources in `osmprj.toml` SHALL be processed. Unknown source names SHALL produce an error and exit non-zero before any downloads begin.

#### Scenario: Sync all sources
- **WHEN** user runs `osmprj sync` with no arguments
- **THEN** all sources defined in `osmprj.toml` are downloaded and imported

#### Scenario: Sync specific sources
- **WHEN** user runs `osmprj sync albania us-alabama`
- **THEN** only the `albania` and `us-alabama` sources are processed

#### Scenario: Unknown source name
- **WHEN** user runs `osmprj sync nonexistent`
- **THEN** the command exits non-zero with an error message naming the unknown source before any downloads start

### Requirement: Sync requires osmprj.toml
`osmprj sync` SHALL exit non-zero with a clear error if `osmprj.toml` is not present in the current directory.

#### Scenario: No project file
- **WHEN** user runs `osmprj sync` in a directory without `osmprj.toml`
- **THEN** the command exits non-zero with a message indicating the project file is missing

### Requirement: Downloads run concurrently
All Geofabrik sources (those without a `path` field) SHALL be downloaded concurrently, subject to the `max_concurrent_downloads` limit (default 3). Each download SHALL display an individual indicatif progress bar showing bytes received, total size, transfer rate, and ETA. Downloads SHALL be ordered largest-first based on pre-flight HEAD requests.

#### Scenario: Concurrent downloads respect cap
- **WHEN** `osmprj.toml` contains five or more Geofabrik sources and `max_concurrent_downloads = 3`
- **THEN** at most 3 downloads run simultaneously at any time

#### Scenario: Local path sources skipped
- **WHEN** a source has a `path` field set
- **THEN** no download is attempted for that source

### Requirement: Imports run concurrently up to configured limit
Sources SHALL be imported into the database concurrently, up to the `max_concurrent_imports` limit (default 1). Each import SHALL begin as soon as its download completes, not after all downloads finish. A failed import SHALL NOT stop other in-flight or pending imports.

#### Scenario: Import starts on download completion
- **WHEN** a source's download and MD5 verification succeed
- **THEN** its import begins immediately without waiting for other downloads

#### Scenario: Import concurrency respects cap
- **WHEN** `max_concurrent_imports = 1` and one import is running
- **THEN** the next ready source waits until the running import finishes

#### Scenario: Import failure does not stop other imports
- **WHEN** source A's import fails
- **THEN** source B's import continues if it is already in-flight or waiting

### Requirement: Failed downloads do not block other downloads
If one download fails, all other in-progress and queued downloads SHALL continue. The import phase SHALL NOT begin for any source whose download failed.

#### Scenario: One of two downloads fails
- **WHEN** source A's download fails and source B's download is in progress
- **THEN** source B's download finishes; source A is not imported; errors are reported at the end

### Requirement: Download-to-import transition message
The separate "files ready — starting imports" transition message SHALL be removed. The unified progress display replaces it. No explicit phase-separator line is printed between downloads and imports.

#### Scenario: No phase separator printed
- **WHEN** a source completes downloading and its import begins
- **THEN** no "files ready" transition line is printed; the spinner appears in the live display

### Requirement: Imports run sequentially
**REMOVED** — superseded by "Imports run concurrently up to configured limit".

**Reason**: The pipelined architecture allows concurrent imports controlled by a semaphore. Sequential execution is now the special case of `max_concurrent_imports = 1` (the default), not a hard constraint.

**Migration**: Set `max_concurrent_imports = 1` in `osmprj.toml` (or omit the field; it defaults to 1) to preserve sequential import behaviour.

### Requirement: Replication initialised per source
After each source's import and post-processing succeed, `osm2pgsql-replication init -d <database_url> --schema <effective_schema>` SHALL be run immediately as part of that source's import task, not in a separate end-of-sync loop. If `osm2pgsql-replication` is not on `PATH`, the command SHALL exit with a clear error.

#### Scenario: Replication init runs immediately after import success
- **WHEN** a source's osm2pgsql import and post-processing SQL complete successfully
- **THEN** `osm2pgsql-replication init` runs for that source before the import semaphore permit is released

#### Scenario: Missing osm2pgsql-replication binary
- **WHEN** `osm2pgsql-replication` is not found on PATH
- **THEN** the command exits non-zero with an error message

### Requirement: Downloads verified with MD5
After each download completes, the tool SHALL fetch `<pbf_url>.md5` from Geofabrik and verify the downloaded file's MD5 hash matches. A mismatch SHALL be treated as a download failure.

#### Scenario: MD5 matches
- **WHEN** the downloaded file's MD5 matches the `.md5` file
- **THEN** the download is recorded as successful in `osmprj.lock`

#### Scenario: MD5 mismatch
- **WHEN** the downloaded file's MD5 does not match the `.md5` file
- **THEN** the download is treated as a failure and reported in the error summary

### Requirement: Skip already-downloaded sources
If `osmprj.lock` contains an entry for a source, the download SHALL be skipped and the existing file used for import. No network request is made for that source's PBF file.

#### Scenario: Source already in lock file
- **WHEN** `osmprj.lock` has an entry for source `albania`
- **THEN** `albania.osm.pbf` is not re-downloaded

### Requirement: Import shows globe spinner
Each import SHALL display an indicatif spinner cycling through globe emojis (🌍 🌎 🌏 🌐) at ~250 ms intervals while `osm2pgsql` is running.

#### Scenario: Spinner visible during import
- **WHEN** `osm2pgsql` is running for a source
- **THEN** a spinner with the source name is shown in the terminal

### Requirement: Import output written to log file
All stdout and stderr from `osm2pgsql` SHALL be written to `<log_dir>/<source_name>.log`. The log directory SHALL be created if it does not exist.

#### Scenario: Log file created
- **WHEN** an import completes
- **THEN** a file `<log_dir>/<source_name>.log` exists containing the full osm2pgsql output

### Requirement: Verbose flag streams import output live
When the `-v` / `--verbose` global flag is set, osm2pgsql stdout and stderr SHALL also be printed to the terminal in addition to the log file.

#### Scenario: Verbose import output
- **WHEN** user runs `osmprj -v sync`
- **THEN** osm2pgsql output is visible in the terminal during import

### Requirement: Partial download file cleanup
When `download_pbf()` fails after creating the destination file (e.g., due to a network error or
connection reset mid-stream), the partial file at `dest` SHALL be deleted before the error is
returned. Failure to delete the partial file SHALL be silently ignored (the file may not exist or
may be locked). This ensures a failed sync leaves no corrupt `.osm.pbf` files in the data
directory.

#### Scenario: Partial file removed on stream error
- **WHEN** the HTTP connection is closed by the server after a partial body is received
- **THEN** `osmprj sync` exits non-zero and no `.osm.pbf` file remains in the data directory

#### Scenario: Cleanup does not mask the original error
- **WHEN** a stream error occurs and the partial file is successfully deleted
- **THEN** the original download error is still returned and reported in stderr

#### Scenario: Cleanup failure is silently ignored
- **WHEN** `remove_file` fails (e.g., file already gone)
- **THEN** the original stream error is returned and no secondary error is reported

### Requirement: Post-processing failures are soft warnings
Post-processing SQL failures — whether caused by a DB connect error or by SQL execution errors — SHALL NOT fail the import task. The import pipeline SHALL continue to the replication init step after a post-processing failure. The failure SHALL be recorded as a warning, printed to stderr, and surfaced in a warning summary after all work completes. Post-processing warnings SHALL NOT cause a non-zero exit code. The final warning summary SHALL suggest re-running with `--postprocess-only` to retry.

#### Scenario: DB connect failure during post-processing warns and continues
- **WHEN** post-processing SQL is configured for a source but the DB is unreachable at that point
- **THEN** a warning is printed to stderr and replication init proceeds normally
- **AND** the command exits zero if no other hard errors occurred

#### Scenario: SQL execution error warns and continues
- **WHEN** a post-processing SQL file contains a statement that fails
- **THEN** a warning is printed to stderr and replication init proceeds normally
- **AND** the command exits zero if no other hard errors occurred

#### Scenario: Post-processing warnings appear in final summary
- **WHEN** one or more sources have post-processing warnings
- **THEN** all warnings are printed after the progress output clears, with a count and a hint to use `--postprocess-only`

#### Scenario: Exit code is zero when only warnings occurred
- **WHEN** all imports and replication inits succeed but post-processing failed for one source
- **THEN** the command exits zero

### Requirement: Single spinner finish per import task
The progress spinner for an import task SHALL be finished exactly once — at the end of the full pipeline (on success or on a hard failure that causes the task to return an error). Intermediate soft failures (post-processing warnings) SHALL update the spinner message but SHALL NOT call `finish_with_message`, leaving the bar active for subsequent pipeline steps.

#### Scenario: Post-process warning does not freeze the spinner
- **WHEN** post-processing fails softly for a source
- **THEN** the spinner continues animating through the replication init step and is only finished when that step completes (or fails hard)

### Requirement: `--postprocess-only` flag
`osmprj sync --postprocess-only` SHALL skip downloading and importing entirely and instead re-run only the post-processing SQL for each in-scope source. `osm2pgsql` and `osm2pgsql-replication` are not required and SHALL NOT be checked when this flag is set. For each source the DB connection is opened independently (per-source). Sources with no post-processing SQL SHALL be skipped with a notice. Any SQL execution or DB connect error is reported per-source and causes a non-zero exit. The optional source-filter positional arguments are honoured: when provided, only those sources are processed.

#### Scenario: Runs SQL only, no download or import
- **WHEN** user runs `osmprj sync --postprocess-only`
- **THEN** no PBF download is attempted and `osm2pgsql` is not invoked
- **AND** post-processing SQL is executed for each source that has it

#### Scenario: Skips sources with no SQL
- **WHEN** a source has no theme SQL and no `extra_sql` configured
- **THEN** that source is skipped with an informational message and counted as successful

#### Scenario: Source filter is honoured
- **WHEN** user runs `osmprj sync --postprocess-only monaco`
- **THEN** only the `monaco` source's post-processing SQL is executed

#### Scenario: Exit non-zero on SQL failure
- **WHEN** a SQL statement in a post-processing file fails during `--postprocess-only`
- **THEN** the error is reported and the command exits non-zero

#### Scenario: Does not require osm2pgsql on PATH
- **WHEN** user runs `osmprj sync --postprocess-only` and `osm2pgsql` is not on PATH
- **THEN** the command proceeds normally (no binary check is performed)
