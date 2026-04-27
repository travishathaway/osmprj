### Requirement: add validates a Geofabrik ID and appends a source block
The system SHALL accept a positional Geofabrik ID argument, validate it against the cached index, and append a `[sources.<id>]` block to `osmprj.toml`.

#### Scenario: Valid Geofabrik ID appended
- **WHEN** the user runs `osmprj add germany --theme shortbread_v1` and "germany" exists in the Geofabrik index
- **THEN** `osmprj.toml` gains a `[sources.germany]` block with `theme = "shortbread_v1"` and the existing file content is preserved

#### Scenario: Invalid Geofabrik ID rejected
- **WHEN** the user runs `osmprj add nonexistent-place`
- **THEN** the system prints an error stating the ID was not found in the Geofabrik index and exits with a non-zero code without modifying `osmprj.toml`

#### Scenario: Default schema derived from ID
- **WHEN** the user runs `osmprj add europe/germany` without `--schema`
- **THEN** the appended block contains `schema = "europe_germany"` (normalized: `/` → `_`)

#### Scenario: Explicit schema overrides default
- **WHEN** the user runs `osmprj add germany --schema de`
- **THEN** the appended block contains `schema = "de"`

### Requirement: add accepts a local file path as an alternative to a Geofabrik ID
The system SHALL accept `--path <file>` and `--name <label>` flags to register a local `.osm.pbf` file as a source, bypassing the Geofabrik index lookup.

#### Scenario: Local file source appended
- **WHEN** the user runs `osmprj add --path /data/region.osm.pbf --name my-region --theme basic`
- **THEN** `osmprj.toml` gains a `[sources.my-region]` block with `path = "/data/region.osm.pbf"` and `theme = "basic"`

#### Scenario: Local file with normalized schema
- **WHEN** the user runs `osmprj add --path /data/file.osm.pbf --name my-region` without `--schema`
- **THEN** the appended block contains `schema = "my_region"`

### Requirement: add requires osmprj.toml to exist
The system SHALL exit with a non-zero code and a helpful error if `osmprj.toml` does not exist in the current directory.

#### Scenario: No project file
- **WHEN** the user runs `osmprj add germany` in a directory without `osmprj.toml`
- **THEN** the system prints an error directing the user to run `osmprj init` first and exits with a non-zero code

### Requirement: add refuses to register a duplicate source name
The system SHALL exit with a non-zero code if a source with the given name already exists in `osmprj.toml`.

#### Scenario: Duplicate source name
- **WHEN** `osmprj.toml` already has a `[sources.germany]` block and the user runs `osmprj add germany`
- **THEN** the system prints an error and exits with a non-zero code without modifying the file
