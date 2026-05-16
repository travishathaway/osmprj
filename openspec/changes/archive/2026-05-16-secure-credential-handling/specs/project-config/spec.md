## MODIFIED Requirements

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
