## MODIFIED Requirements

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
