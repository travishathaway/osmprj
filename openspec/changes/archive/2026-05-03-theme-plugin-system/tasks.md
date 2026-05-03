## 1. Error Variants

- [x] 1.1 Add `PluginThemeNotFound { name: String, searched_paths: Vec<String> }` variant to `OsmprjError` in `src/error.rs` with a `help` message that lists the searched paths and suggests installing a theme package or setting `OSMPRJ_THEME_PATH`
- [x] 1.2 Add `PostProcessFailed { source: String, file: String, message: String }` variant to `OsmprjError` in `src/error.rs`

## 2. Config Model

- [x] 2.1 Add `PostProcessConfig` struct to `src/config.rs` with fields `include_theme_sql: Option<bool>` and `extra_sql: Option<Vec<String>>`
- [x] 2.2 Add `postprocess: Option<PostProcessConfig>` field to `SourceConfig` in `src/config.rs`

## 3. Theme Registry Module

- [x] 3.1 Create `src/theme_registry.rs` with a `ThemeType` enum (`Themepark`, `Flex`) and a `ThemeManifest` struct matching the `theme.toml` fields (`name`, `version`, `description`, `type`, `entry`)
- [x] 3.2 Add a `ThemeEntry` struct to `src/theme_registry.rs` holding the parsed manifest, the resolved theme directory `PathBuf`, the resolved entry `.lua` `PathBuf`, and a sorted `Vec<PathBuf>` of SQL files from the `sql/` subdirectory
- [x] 3.3 Implement `ThemeRegistry` struct with fields: `entries: Vec<ThemeEntry>` and `searched_paths: Vec<PathBuf>`
- [x] 3.4 Implement the three-tier path collection in `ThemeRegistry::search_paths() -> Vec<PathBuf>`:
  - Tier 1: split `OSMPRJ_THEME_PATH` env var using `std::env::split_paths` (if set)
  - Tier 2: `std::env::current_exe()?.parent()?.parent()?.join("share/osmprj/themes")` (exe-relative system path)
  - Tier 3: `dirs::data_dir()?.join("osmprj/themes")` (user-local path)
- [x] 3.5 Implement `ThemeRegistry::build() -> ThemeRegistry`: iterate `search_paths()`, skip non-existent dirs silently, scan each existing dir for subdirectories containing `theme.toml`, parse each manifest with `toml::from_str`, skip and print a warning on parse errors, build `ThemeEntry` for each valid manifest; if two entries share a name the higher-priority (earlier) tier wins
- [x] 3.6 Implement `ThemeRegistry::find(&self, name: &str) -> Option<&ThemeEntry>` lookup by manifest name
- [x] 3.7 Implement `ThemeRegistry::all(&self) -> &[ThemeEntry]` returning all entries
- [x] 3.8 Implement `ThemeRegistry::searched_paths(&self) -> &[PathBuf]` returning the paths that were checked (for error diagnostics)
- [x] 3.9 Declare `pub mod theme_registry;` in `src/main.rs`

## 4. Lua Wrapper Generation for Plugin Themes

- [x] 4.1 Add `generate_lua_wrapper_for_plugin` function to `src/themepark.rs` that accepts a plugin `ThemeEntry` (themepark type), a `topics_config: Option<&TopicsConfig>`, and `schema: &str`, and generates the same wrapper Lua as the existing `generate_lua_wrapper` but using the plugin's entry path and the plugin's directory joined with `lua/` as the additional `package.path` entry (falling back to themepark root if `THEMEPARK_PATH` is set)
- [x] 4.2 For `ThemeType::Flex` plugin themes no wrapper is generated — the entry path is used directly as the `--style` argument; document this in a code comment

## 5. Theme Resolution in Sync Command

- [x] 5.1 In `src/commands/sync.rs`, call `ThemeRegistry::build()` once before the source loop (replacing or supplementing the existing `themepark::find_root()` call)
- [x] 5.2 In the style path resolution block (used in both Phase 3a and Phase 3b), check the plugin registry first: if the theme name matches a plugin entry, resolve the style path from the plugin entry (themepark type → generate wrapper, flex type → use entry path directly); fall back to the existing built-in themepark resolution if not found in the plugin registry
- [x] 5.3 After a successful fresh import (`run_subprocess` returns `Ok`), call the new post-processing function (implemented in task 6) for that source

## 6. Post-Processing SQL Pipeline

- [x] 6.1 Create `async fn run_postprocess(client: &tokio_postgres::Client, source_name: &str, schema: &str, sql_files: &[PathBuf]) -> Result<(), OsmprjError>` in `src/commands/sync.rs` (or a new `src/postprocess.rs` module): for each file, read contents, replace all occurrences of `{schema}` with the actual schema string, split on `;`, execute each non-empty statement via `client.execute(stmt, &[])`, return `PostProcessFailed` on error
- [x] 6.2 In `src/commands/sync.rs` Phase 3b, after a successful import, collect SQL files to run: if `postprocess.include_theme_sql` is `true` (or absent) and the resolved theme is a plugin with SQL files, include the plugin's sorted SQL file list; append any `postprocess.extra_sql` paths resolved relative to the current working directory; skip post-processing entirely if the collected list is empty
- [x] 6.3 Connect to the database (reusing the existing `db_url`) and call `run_postprocess`; print a spinner/progress line per file and a summary on completion; print a warning and continue (do not abort) if `db_url` is empty

## 7. Theme Validation in Add Command

- [x] 7.1 In `src/commands/add.rs`, when `--theme` is provided, call `ThemeRegistry::build()` and check: if the name is not in the plugin registry AND not in the built-in `theme_config_map`, return `PluginThemeNotFound` with the list of `registry.searched_paths()` formatted as strings

## 8. Themes Subcommand

- [x] 8.1 Create `src/commands/themes.rs` with a `run_list() -> Result<(), OsmprjError>` function that: builds `ThemeRegistry`, prints plugin themes (name, type, description, version, path, SQL file names if any) grouped by source path, then prints the built-in themepark theme names as a flat list
- [x] 8.2 Expose `pub mod themes;` in `src/commands/mod.rs`
- [x] 8.3 Add a `Themes` variant to `Commands` in `src/main.rs` with a `List` sub-subcommand (using `#[command(subcommand)]`)
- [x] 8.4 Route `Commands::Themes(ThemesCommands::List)` to `commands::themes::run_list()` in the `main` match block; this command does not require a project file

## 9. Integration & Unit Tests

- [x] 9.1 Add unit tests in `src/theme_registry.rs` (behind `#[cfg(test)]`): create a temp directory tree with one valid `theme.toml` (themepark type) and one (flex type), call `ThemeRegistry::build()` with `OSMPRJ_THEME_PATH` set to the temp dir, assert both entries are found with correct fields
- [x] 9.2 Add a unit test for the priority/shadowing behaviour: two tiers both provide a theme with the same name; assert the higher-priority tier's entry is returned by `find`
- [x] 9.3 Add a unit test for malformed `theme.toml` (missing required field): assert the registry skips it without panicking
- [x] 9.4 Add a unit test for `run_postprocess` (or the SQL collection logic): given a list of SQL file paths with `{schema}` placeholders and a mock schema name, assert the substitution produces the correct SQL strings
- [x] 9.5 In `tests/integration/test_add.py`, add a test that `osmprj add --theme nonexistent-theme` exits non-zero and prints an error mentioning the theme name and at least one searched path
- [x] 9.6 In `tests/integration/test_themes.py` (new file), add a test that `osmprj themes list` exits zero and prints the built-in theme names (`shortbread_v1`, `basic`, etc.)

## 10. Verification

- [x] 10.1 Run `cargo build` and confirm it compiles without errors or warnings
- [x] 10.2 Run `cargo clippy` and address any lint warnings
- [x] 10.3 Run `cargo test` and confirm all unit tests pass
- [x] 10.4 Manually create a minimal test theme directory (a `theme.toml` + stub `.lua` file), point `OSMPRJ_THEME_PATH` at its parent, and confirm `osmprj themes list` shows it
- [x] 10.5 Manually confirm `osmprj add --theme <nonexistent>` prints the `PluginThemeNotFound` diagnostic with searched paths
- [x] 10.6 Run `pytest tests/integration` and confirm existing and new integration tests pass