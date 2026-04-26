## ADDED Requirements

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
All Geofabrik sources (those without a `path` field) SHALL be downloaded concurrently. Each download SHALL display an individual indicatif progress bar showing bytes received, total size, transfer rate, and ETA.

#### Scenario: Concurrent downloads
- **WHEN** `osmprj.toml` contains two or more Geofabrik sources
- **THEN** both downloads begin simultaneously and each shows its own progress bar

#### Scenario: Local path sources skipped
- **WHEN** a source has a `path` field set
- **THEN** no download is attempted for that source

### Requirement: Downloads verified with MD5
After each download completes, the tool SHALL fetch `<pbf_url>.md5` from Geofabrik and verify the downloaded file's MD5 hash matches. A mismatch SHALL be treated as a download failure.

#### Scenario: MD5 matches
- **WHEN** the downloaded file's MD5 matches the `.md5` file
- **THEN** the download is recorded as successful in `osmprj.lock`

#### Scenario: MD5 mismatch
- **WHEN** the downloaded file's MD5 does not match the `.md5` file
- **THEN** the download is treated as a failure and reported in the error summary

### Requirement: Failed downloads do not block other downloads
If one download fails, all other in-progress downloads SHALL complete before the command reports errors and exits. The import phase SHALL NOT begin if any download failed.

#### Scenario: One of two downloads fails
- **WHEN** source A's download fails and source B's download is in progress
- **THEN** source B's download finishes, both errors are reported, and no imports are started

### Requirement: Skip already-downloaded sources
If `osmprj.lock` contains an entry for a source, the download SHALL be skipped and the existing file used for import. No network request is made for that source's PBF file.

#### Scenario: Source already in lock file
- **WHEN** `osmprj.lock` has an entry for source `albania`
- **THEN** `albania.osm.pbf` is not re-downloaded

### Requirement: Download-to-import transition message
After all downloads complete successfully, all progress bars SHALL be cleared and a single summary line printed containing a map emoji before imports begin.

#### Scenario: Transition message
- **WHEN** all downloads succeed
- **THEN** a line containing `🗺` and a "downloads complete" message is printed before the first import starts

### Requirement: Imports run sequentially
Sources SHALL be imported into the database one at a time in the order they appear in `osmprj.toml`. A failed import SHALL stop all remaining imports immediately.

#### Scenario: Import failure stops further imports
- **WHEN** the import for source A fails
- **THEN** source B is not imported and the command exits non-zero

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

### Requirement: Replication initialised per source
After all imports succeed, `osm2pgsql-replication init -d <database_url> --schema <effective_schema>` SHALL be run once per successfully imported source. If `osm2pgsql-replication` is not on `PATH`, the command SHALL exit with a clear error.

#### Scenario: Replication init runs after successful imports
- **WHEN** all sources are imported successfully
- **THEN** `osm2pgsql-replication init` is called for each source with the correct `--schema`

#### Scenario: Missing osm2pgsql-replication binary
- **WHEN** `osm2pgsql-replication` is not found on PATH
- **THEN** the command exits non-zero with an error message before the import phase
