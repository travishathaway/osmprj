## ADDED Requirements

### Requirement: Concurrent download limit is configurable
The system SHALL read `max_concurrent_downloads` from the `[project]` section of `osmprj.toml`. When absent, the effective value SHALL be 3. The value SHALL be a positive integer. Download concurrency SHALL be enforced via a `tokio::sync::Semaphore` with that number of permits; no more than that many PBF downloads SHALL be in-flight simultaneously.

#### Scenario: Default concurrent downloads
- **WHEN** `osmprj.toml` has no `max_concurrent_downloads` field
- **THEN** at most 3 PBF downloads run simultaneously

#### Scenario: Custom concurrent downloads
- **WHEN** `osmprj.toml` sets `max_concurrent_downloads = 2`
- **THEN** at most 2 PBF downloads run simultaneously

#### Scenario: Single download at a time
- **WHEN** `osmprj.toml` sets `max_concurrent_downloads = 1`
- **THEN** downloads run one at a time in sorted order

### Requirement: Concurrent import limit is configurable
The system SHALL read `max_concurrent_imports` from the `[project]` section of `osmprj.toml`. When absent, the effective value SHALL be 1. The value SHALL be a positive integer. Import concurrency SHALL be enforced via a `tokio::sync::Semaphore` with that number of permits; no more than that many osm2pgsql import pipelines (including post-processing and replication init) SHALL be in-flight simultaneously.

#### Scenario: Default concurrent imports
- **WHEN** `osmprj.toml` has no `max_concurrent_imports` field
- **THEN** at most 1 import runs at a time (sequential, preserving current behaviour)

#### Scenario: Custom concurrent imports
- **WHEN** `osmprj.toml` sets `max_concurrent_imports = 2`
- **THEN** at most 2 imports may run simultaneously

### Requirement: File sizes probed before downloading
Before seeding downloads, the system SHALL issue one HTTP HEAD request per source that requires downloading (fresh, no lock entry, no local `path`). The `Content-Length` response header SHALL be used as the file size. All HEAD requests SHALL be issued concurrently. A HEAD request failure or missing `Content-Length` SHALL NOT cause the sync to fail; that source's size SHALL be treated as 0 bytes for sorting purposes.

#### Scenario: HEAD request returns Content-Length
- **WHEN** a HEAD request to a Geofabrik PBF URL returns a `Content-Length` header
- **THEN** that value is recorded as the source's file size for sorting

#### Scenario: HEAD request fails
- **WHEN** a HEAD request fails or returns no `Content-Length`
- **THEN** that source is treated as 0 bytes for sort order and the sync continues normally

### Requirement: Sources downloaded largest-first
After the pre-flight size probe, sources SHALL be sorted by file size in descending order (largest first). Downloads SHALL be seeded into the semaphore queue in this order.

#### Scenario: Largest file downloads first
- **WHEN** sources have sizes 100 MB, 500 MB, and 200 MB
- **THEN** the 500 MB source is queued first, then 200 MB, then 100 MB
