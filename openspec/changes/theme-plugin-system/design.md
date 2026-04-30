## Context

`osmprj` is a Rust CLI tool that orchestrates osm2pgsql imports and replication for OpenStreetMap data. The existing theme system in `src/themepark.rs` is entirely static: a hardcoded match table maps theme names to `.lua` filenames inside a themepark installation found via `THEMEPARK_PATH`. There is no extension point, no post-processing pipeline, and no way to distribute themes as packages.

The codebase uses `tokio` (async), `tokio-postgres` (DB), `clap` (CLI derive), `serde`/`toml` (config), `miette`/`thiserror` (errors), and `dirs` (cross-platform user paths). All commands follow: validate → load config → side effects → print results.

## Goals / Non-Goals

**Goals:**
- Discover theme packages installed into standard filesystem locations without any hardcoded package-manager paths.
- Define a stable on-disk layout and manifest format for theme packages.
- Support both themepark-backed and pure flex Lua themes transparently.
- Run numbered post-processing SQL files once after a fresh import, with `{schema}` substitution.
- Validate theme existence at `osmprj add` time with actionable diagnostics.
- Surface installed themes via `osmprj themes list`.

**Non-Goals:**
- Migrating built-in themepark themes to the new package format (deferred).
- Re-running post-processing SQL on replication updates.
- A package manager, version resolution, or dependency graph for themes.
- SQL templating beyond `{schema}` substitution.

## Decisions

### 1. Theme discovery via executable-relative path, not package-manager env vars

Theme search paths are derived at runtime without hardcoding any package-manager-specific environment variable (`CONDA_PREFIX`, `VIRTUAL_ENV`, etc.). This ensures osmprj works correctly regardless of how it was installed (conda, Homebrew, apt, manual, Windows installer).

The search strategy uses three tiers, checked in priority order (highest first):

```
Tier 1: OSMPRJ_THEME_PATH env var (colon-separated on Unix, semicolon on Windows)
         — escape hatch for power users and CI

Tier 2: <exe_prefix>/share/osmprj/themes/
         — system/package-manager install
         resolved as: std::env::current_exe()?.parent()?.parent()?.join("share/osmprj/themes")
         Examples:
           /opt/conda/envs/foo/bin/osmprj  →  /opt/conda/envs/foo/share/osmprj/themes/
           /opt/homebrew/bin/osmprj        →  /opt/homebrew/share/osmprj/themes/
           /usr/bin/osmprj                 →  /usr/share/osmprj/themes/
           C:\tools\bin\osmprj.exe         →  C:\tools\share\osmprj\themes\

Tier 3: dirs::data_dir()/osmprj/themes/
         — user-local install (no elevated privileges needed)
         Linux:   ~/.local/share/osmprj/themes/
         macOS:   ~/Library/Application Support/osmprj/themes/
         Windows: C:\Users\<user>\AppData\Roaming\osmprj\themes\
```

All existing (non-error) directories from all tiers are scanned. Each subdirectory containing a `theme.toml` is treated as a theme package. If two tiers provide a theme with the same name, the higher-priority tier wins.

*Alternative considered:* `CONDA_PREFIX/share/osmprj/themes/`. Rejected — ties the tool to a specific package manager and breaks non-conda installs.

*Alternative considered:* A global registry file listing installed themes. Rejected — unnecessary indirection; filesystem scanning is simpler, more robust, and how most Unix tools work.

### 2. Theme package on-disk layout

```
<themes_root>/
└── <theme-name>/
    ├── theme.toml          required — manifest
    ├── <entry>.lua         required — Lua entry point named in manifest
    └── sql/                optional — post-processing SQL
        ├── 01_indexes.sql
        └── 02_views.sql
```

`theme.toml` manifest fields:

```toml
name        = "shortbread-v2"          # must match directory name
version     = "2.0.1"
description = "Shortbread tile schema v2"
type        = "themepark"              # "themepark" | "flex"
entry       = "shortbread-v2.lua"     # relative to theme directory
```

`type = "themepark"`: osmprj generates a Lua wrapper that sets `package.path`, calls `require('themepark')`, sets the schema option, and either enumerates topics or `dofile`s the entry. Identical to how built-in themepark themes are handled today, but the entry path comes from the plugin directory rather than from `THEMEPARK_PATH/config/`.

`type = "flex"`: the entry `.lua` file is passed directly to `osm2pgsql --style=` without any wrapping. The user-supplied Lua script is responsible for all schema handling.

### 3. SQL post-processing runs once after fresh import only

SQL files in a theme's `sql/` directory are executed once, in ascending filename order, immediately after a successful `osm2pgsql` import. They do not run on replication updates (`osm2pgsql-replication update`).

This is intentional: these files are for schema enrichment (indexes, derived tables, views) that is set up once. Replication updates apply incremental diffs; the enriched objects persist.

SQL files support a single template variable: `{schema}` is replaced with the source's effective schema name before execution. No other substitution is performed.

Execution uses the existing `tokio-postgres` client. Each file is executed as a single multi-statement query. Failure aborts the post-processing phase and returns `PostProcessFailed`.

### 4. `[postprocess]` config block — opt-out of theme SQL, opt-in to extra SQL

```toml
[sources.germany]
theme  = "shortbread-v2"
schema = "germany"

# optional block — theme SQL runs by default when present
[sources.germany.postprocess]
include_theme_sql = true          # default: true
extra_sql = ["./local/extra.sql"] # relative to osmprj.toml directory
```

When no `[postprocess]` block is present, theme SQL (if any) runs automatically. `include_theme_sql = false` suppresses theme SQL. `extra_sql` paths are resolved relative to the directory containing `osmprj.toml` and appended after theme SQL.

### 5. Theme validation at `osmprj add` time

`osmprj add --theme <name>` calls `ThemeRegistry::find(name)` before writing to `osmprj.toml`. If the theme is not found in the plugin registry and is not a built-in themepark theme, the command fails immediately with `ThemeNotFound`, which includes the list of all searched paths in its diagnostic message:

```
error: theme 'shortbread-v2' not found

  osmprj searched:
    /opt/conda/envs/foo/share/osmprj/themes/   (no themes found)
    ~/.local/share/osmprj/themes/              (directory does not exist)

  Install a theme package or set OSMPRJ_THEME_PATH.
```

*Alternative considered:* Lazy validation at sync time. Rejected — fail-fast is better UX and catches typos before they reach a long-running import.

### 6. `osmprj themes list` subcommand

A new `themes` top-level subcommand with a `list` sub-subcommand prints all discovered plugin themes grouped by source path, followed by the built-in themepark themes. Output is human-readable plain text (no JSON format at this stage).

```
$ osmprj themes list

  PLUGIN THEMES

  shortbread-v2     themepark   "Shortbread vector tile schema v2"  v2.0.1
  /opt/conda/envs/foo/share/osmprj/themes/shortbread-v2
  sql: 01_indexes.sql, 02_views.sql

  BUILT-IN (themepark)
  shortbread_v1, shortbread_v1_gen, basic, generic, osmcarto, experimental
```

### 7. New module: `src/theme_registry.rs`

All plugin theme discovery and manifest parsing is isolated in a new `theme_registry.rs` module exposing:

- `ThemeEntry`: holds manifest fields + resolved paths (theme dir, entry lua path, sql files sorted).
- `ThemeRegistry`: the in-memory collection built by scanning search paths.
- `ThemeRegistry::build() -> ThemeRegistry`: performs the three-tier directory scan, skips unreadable dirs without error, logs warnings for malformed `theme.toml` files.
- `ThemeRegistry::find(name: &str) -> Option<&ThemeEntry>`: lookup by name.
- `ThemeRegistry::all() -> &[ThemeEntry]`: for listing.
- `ThemeRegistry::searched_paths() -> &[PathBuf]`: for error diagnostics.

`ThemeRegistry::build()` is called once at the start of `sync` and `add` commands. It is not persisted or cached across invocations.

## Risks / Trade-offs

- **`current_exe()` symlink resolution**: `std::env::current_exe()` follows symlinks on most platforms, returning the real binary path. If osmprj is invoked via a symlink in a different directory tree (e.g. `/usr/local/bin/osmprj → /opt/tools/bin/osmprj`), the share path is derived from `/opt/tools/share/osmprj/themes/`, which is correct. The `OSMPRJ_THEME_PATH` override covers edge cases.
- **Dev builds (`cargo run`)**: The exe is at `target/debug/osmprj`, so the system path resolves to `target/share/osmprj/themes/` — nonexistent in a dev checkout. Developers must use `OSMPRJ_THEME_PATH` to point at test theme fixtures. This is documented.
- **Windows path separators**: `OSMPRJ_THEME_PATH` uses `;` as separator on Windows (consistent with `PATH`). The code uses `std::env::split_paths` which handles this automatically.
- **Malformed `theme.toml`**: Missing or unparseable manifests are skipped with a warning rather than failing the entire build — a single broken package should not prevent all other themes from loading.
- **Multi-statement SQL**: `tokio-postgres` does not support multi-statement queries via the standard `query` API. SQL files will be split on `;` boundaries and each statement executed individually, skipping empty statements. This is a known limitation and is documented in error messages.
