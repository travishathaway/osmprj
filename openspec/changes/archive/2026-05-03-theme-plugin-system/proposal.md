## Why

`osmprj` currently supports a fixed set of hardcoded themes from `osm2pgsql-themepark`. Adding a new theme requires modifying the source code. There is no way for users or third parties to install additional Lua flex styles or post-processing SQL without forking the project. This limits the ecosystem and makes it impossible to distribute domain-specific import styles (e.g. for public transport, cycling infrastructure, or address databases) as standalone packages.

Additionally, there is no mechanism for running SQL after a fresh import. Users who need to create derived tables, add custom indexes, or build materialized views must do so manually after every initial sync.

## What Changes

- Introduce a **plugin theme system** that discovers theme packages installed into well-known filesystem paths, derived from the location of the `osmprj` binary (i.e. `<exe_prefix>/share/osmprj/themes/`), plus a user-local data directory and an optional `OSMPRJ_THEME_PATH` override.
- Define a **theme package layout**: a directory containing a `theme.toml` manifest, a Lua entry point, and an optional `sql/` subdirectory of numbered post-processing SQL files.
- Support two theme types in `theme.toml`: `themepark` (osmprj generates the schema-injecting Lua wrapper, as today) and `flex` (the Lua file is passed directly to osm2pgsql with no wrapping).
- Add a **post-processing SQL pipeline** that runs numbered SQL files from the theme's `sql/` directory (and optionally user-supplied extra SQL files) once after a fresh import. SQL files support `{schema}` template substitution.
- Extend `SourceConfig` with an optional `[postprocess]` block (`include_theme_sql`, `extra_sql`) giving users control over which SQL runs.
- Add an `osmprj themes list` subcommand that shows all discovered plugin themes and the built-in themepark themes.
- Validate that a referenced theme exists (plugin or built-in) at `osmprj add` time, with a diagnostic error that shows all searched paths.

## Capabilities

### New Capabilities

- `theme-discovery`: Scan well-known paths (exe-relative system path, user data dir, env var override) to build an in-memory registry of installed plugin themes at startup.
- `theme-plugin`: Support externally installed theme packages with a `theme.toml` manifest declaring type (`themepark` | `flex`), entry Lua file, description, and version.
- `sql-postprocess`: After a fresh osm2pgsql import, execute numbered SQL files from the theme package and/or user-specified paths, with `{schema}` substitution, via tokio-postgres.
- `themes-subcommand`: New `osmprj themes list` CLI command displaying discovered plugin themes and built-in themes.

### Modified Capabilities

- `theme-resolution`: `osmprj add --theme` and the sync pipeline now check the plugin theme registry first before falling back to the existing built-in themepark resolution. Theme existence is validated at `add` time.

## Testing

Integration and unit tests will cover:

- Theme discovery correctly scanning all three path tiers (exe-relative, user data dir, env var override), merging results, and applying priority order.
- `theme.toml` manifest parsing for both `themepark` and `flex` types, and graceful handling of malformed or missing manifests.
- Lua wrapper generation for `themepark`-type plugin themes (schema injection, topics) using the same path as existing built-in themes.
- Direct style path pass-through for `flex`-type plugin themes.
- SQL post-processing: numbered file ordering, `{schema}` substitution, and execution only on fresh import (not on replication update).
- `osmprj add --theme <name>` fails with a clear diagnostic when the theme does not exist, listing the searched paths.
- `osmprj themes list` output includes discovered plugin themes and built-in themes.
- `[postprocess] include_theme_sql = false` suppresses theme SQL; `extra_sql` paths are executed in addition to (or instead of) theme SQL.

## Impact

- `src/theme_registry.rs`: New module implementing path discovery, manifest parsing, and the in-memory theme registry.
- `src/themepark.rs`: Extended to accept plugin theme paths in addition to the existing built-in resolution. `generate_lua_wrapper` gains a path-based entry for plugin themepark themes.
- `src/config.rs`: New `PostProcessConfig` struct; `SourceConfig` gains `postprocess: Option<PostProcessConfig>`.
- `src/commands/sync.rs`: New Phase 4 post-processing step after fresh imports; theme resolution updated to consult plugin registry.
- `src/commands/add.rs`: Theme validation at add time using the plugin registry.
- `src/commands/themes.rs`: New file implementing `osmprj themes list`.
- `src/commands/mod.rs`: Expose the new `themes` module.
- `src/main.rs`: Add `Themes` subcommand with `list` subcommand.
- `src/error.rs`: New error variants: `ThemeNotFound` (with searched paths), `PostProcessFailed`.
- No new Cargo dependencies required.
