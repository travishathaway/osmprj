## ADDED Requirements

### Requirement: Import begins immediately when download completes
The system SHALL start importing a source as soon as its download and MD5 verification complete, without waiting for other downloads to finish. The import SHALL be submitted to the import semaphore immediately upon download success.

#### Scenario: Small source imports while large source downloads
- **WHEN** source A (100 MB) finishes downloading while source B (1 GB) is still downloading
- **THEN** source A's import begins immediately, running concurrently with source B's download

#### Scenario: Import semaphore blocks when at capacity
- **WHEN** `max_concurrent_imports = 1` and an import is already running
- **THEN** the next completed download's import waits until the running import finishes before starting

### Requirement: Per-source import pipeline runs atomically within semaphore
Each source's import pipeline — osm2pgsql import, post-processing SQL, and replication init — SHALL run as a single unit holding the import semaphore permit. The permit SHALL NOT be released between sub-phases.

#### Scenario: Import semaphore held through replication init
- **WHEN** osm2pgsql finishes for source A and post-processing begins
- **THEN** the import semaphore permit is still held and no new import can start (when at capacity)

### Requirement: Replication init runs immediately after import success
`osm2pgsql-replication init` SHALL be called for each source immediately after its post-processing SQL completes successfully, within the same import task. The separate end-of-sync replication loop SHALL be removed.

#### Scenario: Replication init follows import without delay
- **WHEN** source A's osm2pgsql import and post-processing succeed
- **THEN** `osm2pgsql-replication init` runs for source A before the import semaphore is released

### Requirement: All errors collected before reporting
Download and import errors SHALL be accumulated across all concurrent tasks. The sync command SHALL NOT abort on the first failure. After the unified event loop completes, all errors SHALL be printed together and the command SHALL exit non-zero if any errors occurred. Successfully completed sources SHALL remain in `osmprj.lock`.

#### Scenario: Import failure does not abort other imports
- **WHEN** source A's import fails and source B's import is in-flight
- **THEN** source B's import continues to completion before errors are reported

#### Scenario: Mixed success and failure reported together
- **WHEN** source A succeeds and source B fails
- **THEN** source A is recorded in osmprj.lock, source B's error is printed, and the command exits non-zero

#### Scenario: All errors printed at end
- **WHEN** two sources fail (one download, one import)
- **THEN** both error messages are printed after all in-flight work completes

### Requirement: Unified progress display during pipeline
A single `MultiProgress` instance SHALL render both download progress bars and import spinners simultaneously. Download tasks SHALL display byte-progress bars. Import tasks SHALL display a spinner whose message updates through sub-phases. Completed items SHALL print a final status line and scroll out of the live display.

#### Scenario: Download and import visible simultaneously
- **WHEN** source A is importing and source B is downloading
- **THEN** both A's spinner and B's progress bar are visible in the terminal at the same time

#### Scenario: Import spinner message progresses through sub-phases
- **WHEN** an import task runs
- **THEN** the spinner message reads "Importing {name}..." during osm2pgsql, "Post-processing {name}..." during SQL execution, and "Initialising replication for {name}..." during replication init

#### Scenario: Completed items leave final status line
- **WHEN** a download or import completes
- **THEN** a final "✓ {name} imported" (or equivalent) line is printed above the live display
