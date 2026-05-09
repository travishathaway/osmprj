## ADDED Requirements

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
