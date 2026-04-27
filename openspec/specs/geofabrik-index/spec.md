### Requirement: Index is fetched from Geofabrik and cached permanently
The system SHALL download `https://download.geofabrik.de/index-v1.json` on first use and store it at `<OS cache dir>/osmprj/geofabrik-index-v1.json`. Subsequent calls SHALL read from the cache without making a network request.

#### Scenario: First use fetches and caches
- **WHEN** the cache file does not exist and `geofabrik::load_index()` is called
- **THEN** the system downloads the index, writes it to the cache path, and returns the parsed index

#### Scenario: Subsequent use reads from cache
- **WHEN** the cache file already exists and `geofabrik::load_index()` is called
- **THEN** the system reads from disk without making a network request and returns the parsed index

#### Scenario: Cache directory is created if absent
- **WHEN** the `<OS cache dir>/osmprj/` directory does not exist
- **THEN** the system creates it before writing the cache file

### Requirement: Index lookup resolves a Geofabrik ID to a PBF download URL
The system SHALL provide a `lookup(id: &str, index: &GeofabrikIndex) -> Option<&str>` function that returns the `urls.pbf` value for a matching feature.

#### Scenario: Known ID resolves
- **WHEN** `lookup("germany", &index)` is called and "germany" exists in the index
- **THEN** the function returns `Some` containing the PBF URL for germany

#### Scenario: Unknown ID returns None
- **WHEN** `lookup("nonexistent-place", &index)` is called
- **THEN** the function returns `None`

### Requirement: Network failure produces a clear error
The system SHALL return a descriptive error when the index cannot be fetched, rather than panicking.

#### Scenario: Network unavailable
- **WHEN** the cache file does not exist and the network request fails
- **THEN** `geofabrik::load_index()` returns an `Err` with a message indicating the fetch failed
