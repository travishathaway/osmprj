### Requirement: ProjectConfig represents the full osmprj.toml schema
The system SHALL provide a `ProjectConfig` struct in `src/config.rs` that deserializes a complete `osmprj.toml` file, including top-level fields and all `[sources.*]` sections.

#### Scenario: Full config round-trip
- **WHEN** `osmprj.toml` contains `database_url`, multiple `[sources.*]` blocks each with `theme` and optional `schema`, and a `[sources.*.topics]` sub-table
- **THEN** `ProjectConfig::load()` deserializes all fields without error and all values are accessible on the resulting struct

#### Scenario: Empty file loads with defaults
- **WHEN** `osmprj.toml` exists but contains no keys
- **THEN** `ProjectConfig::load()` returns a config with `database_url = None`, `database_url_command = None`, and an empty sources map

### Requirement: ProjectSettings includes database_url_command field
The system SHALL provide a `database_url_command: Option<String>` field on `ProjectSettings` that deserializes from the `database_url_command` key in the `[project]` section of `osmprj.toml`.

#### Scenario: Command field present
- **WHEN** `osmprj.toml` contains `database_url_command = "pass show osmprj/db"`
- **THEN** `config.project.database_url_command` is `Some("pass show osmprj/db")`

#### Scenario: Command field absent
- **WHEN** `osmprj.toml` does not contain `database_url_command`
- **THEN** `config.project.database_url_command` is `None`

#### Scenario: Both database_url and database_url_command present
- **WHEN** `osmprj.toml` contains both `database_url` and `database_url_command`
- **THEN** `ProjectConfig::load()` deserializes both fields without error; resolution order is handled at runtime by `effective_database_url()`

### Requirement: SourceConfig models a single data source
The system SHALL provide a `SourceConfig` struct with fields: `path: Option<String>`, `theme: Option<String>`, `schema: Option<String>`, and `topics: Option<TopicsConfig>`.

#### Scenario: Geofabrik source (no path)
- **WHEN** a source block has `theme = "shortbread_v1"` and no `path` key
- **THEN** `source.path` is `None` and `source.theme` is `Some("shortbread_v1")`

#### Scenario: Local file source (with path)
- **WHEN** a source block has `path = "/data/file.osm.pbf"` and `theme = "basic"`
- **THEN** `source.path` is `Some("/data/file.osm.pbf")` and `source.theme` is `Some("basic")`

### Requirement: TopicsConfig models topic customization
The system SHALL provide a `TopicsConfig` struct with fields: `list: Option<Vec<String>>`, `add: Option<Vec<String>>`, `remove: Option<Vec<String>>`.

#### Scenario: Theme override with add and remove
- **WHEN** a `[sources.x.topics]` block contains `add = ["basic/generic-points"]` and `remove = ["shortbread_v1/aerialways"]`
- **THEN** `topics.add` and `topics.remove` contain the respective lists and `topics.list` is `None`

#### Scenario: Explicit topic list
- **WHEN** a `[sources.x.topics]` block contains only a `list` array
- **THEN** `topics.list` contains the array and `topics.add` and `topics.remove` are both `None`

### Requirement: Schema name normalization
The system SHALL compute a default schema name from the source name by replacing all `/` and `-` characters with `_` when the `schema` field is not explicitly set.

#### Scenario: Geofabrik ID with slash
- **WHEN** a source is named `europe/germany` and has no explicit `schema`
- **THEN** the effective schema name is `europe_germany`

#### Scenario: Hyphenated source name
- **WHEN** a source is named `north-america` and has no explicit `schema`
- **THEN** the effective schema name is `north_america`

#### Scenario: Explicit schema overrides normalization
- **WHEN** a source named `europe/germany` has `schema = "de"`
- **THEN** the effective schema name is `de`
