## 1. Dependencies

- [x] 1.1 Add `reqwest` (features: `blocking`, `json`) to `Cargo.toml`
- [x] 1.2 Add `dirs` to `Cargo.toml`
- [x] 1.3 Add `toml_edit` to `Cargo.toml`

## 2. Project Config Model

- [x] 2.1 Replace `src/config.rs` with `ProjectConfig` struct containing `database_url: Option<String>` and `sources: HashMap<String, SourceConfig>`
- [x] 2.2 Add `SourceConfig` struct with `path`, `theme`, `schema` (`Option<String>`) and `topics: Option<TopicsConfig>`
- [x] 2.3 Add `TopicsConfig` struct with `list`, `add`, `remove` (`Option<Vec<String>>`)
- [x] 2.4 Implement `ProjectConfig::load()` reading `osmprj.toml` from CWD; return `Ok(None)` when file is absent
- [x] 2.5 Implement `SourceConfig::effective_schema(name: &str) -> String` — returns explicit `schema` or normalizes the name (replace `/` and `-` with `_`)

## 3. Geofabrik Index Module

- [x] 3.1 Create `src/geofabrik.rs` with a `GeofabrikFeature` struct capturing at minimum `id: String` and `urls.pbf: Option<String>` from the GeoJSON properties
- [x] 3.2 Implement `load_index() -> Result<Vec<GeofabrikFeature>, Box<dyn Error>>` — reads from cache if present, otherwise fetches from `https://download.geofabrik.de/index-v1.json`, writes to `<cache_dir>/osmprj/geofabrik-index-v1.json`, and parses
- [x] 3.3 Implement `lookup<'a>(id: &str, features: &'a [GeofabrikFeature]) -> Option<&'a GeofabrikFeature>` — case-sensitive match on `id` field

## 4. Command Module Structure

- [x] 4.1 Create `src/commands/mod.rs` re-exporting `init` and `add` modules
- [x] 4.2 Create empty `src/commands/init.rs` and `src/commands/add.rs`
- [x] 4.3 Update `src/main.rs` to declare `mod commands` and route `Commands::Init` and `Commands::Add` through the new modules; add argument structs to both variants

## 5. Init Command

- [x] 5.1 Add `--db <url>` optional argument to the `Init` clap variant
- [x] 5.2 Implement `commands::init::run(db: Option<String>)` — error if `osmprj.toml` exists; write file with `database_url` (if provided) and a `# add sources with: osmprj add` comment
- [x] 5.3 Print a confirmation message on success (e.g., `Created osmprj.toml`)

## 6. Add Command

- [x] 6.1 Add positional `geofabrik_id: Option<String>` and flags `--path`, `--name`, `--theme`, `--schema` to the `Add` clap variant; enforce that either a positional ID or `--path`+`--name` is provided
- [x] 6.2 Implement `commands::add::run(...)` — error if `osmprj.toml` does not exist
- [x] 6.3 Implement Geofabrik path: call `geofabrik::load_index()`, validate ID via `lookup()`, error with message if not found
- [x] 6.4 Implement local file path: skip index lookup when `--path` is provided
- [x] 6.5 Compute effective schema via `SourceConfig::effective_schema(name)`
- [x] 6.6 Load existing `osmprj.toml` via `toml_edit`, error if source name already exists, append new `[sources.<name>]` table, write file back
- [x] 6.7 Print a confirmation message on success (e.g., `Added [sources.germany] to osmprj.toml`)

## 7. Smoke Tests

- [x] 7.1 Build with `pixi run --environment rust build-rust` and confirm no errors
- [x] 7.2 Run `osmprj init --db postgres://localhost/osm` and inspect the generated `osmprj.toml`
- [x] 7.3 Run `osmprj add germany --theme shortbread_v1` and confirm the source block appears in `osmprj.toml`
- [x] 7.4 Run `osmprj add germany` a second time and confirm a duplicate-source error is printed
- [x] 7.5 Run `osmprj add nonexistent-id` and confirm an unknown-ID error is printed
