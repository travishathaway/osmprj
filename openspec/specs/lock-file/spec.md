## ADDED Requirements

### Requirement: Lock file records download metadata per source
`osmprj.lock` SHALL be a TOML file in the project root with a `[sources.<name>]` section for each downloaded source, containing `url`, `md5`, and `downloaded_at` (RFC 3339 timestamp). Local `--path` sources SHALL NOT have lock entries.

#### Scenario: Lock entry after download
- **WHEN** source `albania` is successfully downloaded
- **THEN** `osmprj.lock` contains `[sources.albania]` with `url`, `md5`, and `downloaded_at`

#### Scenario: Local path source has no lock entry
- **WHEN** source has `path = "/data/region.pbf"`
- **THEN** no entry for that source appears in `osmprj.lock`

### Requirement: Lock file records themepark cache state
`osmprj.lock` SHALL contain a `[themepark]` section with `cached_at` (RFC 3339 timestamp) after themepark is downloaded.

#### Scenario: Themepark section present after download
- **WHEN** themepark is downloaded and cached
- **THEN** `osmprj.lock` contains `[themepark]` with a `cached_at` value

### Requirement: Lock written after each successful download
The lock file SHALL be updated on disk immediately after each source's download and MD5 verification succeeds, not deferred to end-of-sync. If sync is interrupted after two of three downloads, the lock records those two.

#### Scenario: Incremental lock writes
- **WHEN** source A downloads successfully before source B completes
- **THEN** `osmprj.lock` contains source A's entry even if source B subsequently fails

### Requirement: Lock file preserved across syncs
On subsequent `osmprj sync` runs, the existing lock file SHALL be read at startup. Existing entries for skipped (already-downloaded) sources SHALL be preserved unchanged.

#### Scenario: Re-sync preserves existing lock entries
- **WHEN** `osmprj sync` is run and some sources are skipped due to existing lock entries
- **THEN** those entries remain intact in `osmprj.lock` after sync completes
