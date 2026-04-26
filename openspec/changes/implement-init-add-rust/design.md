## Context

The Rust binary (`src/main.rs`) has a working CLI skeleton with five placeholder commands. `src/config.rs` holds a minimal `Settings` struct with only `database_url`. This change replaces that stub with a full project config model and implements the two commands that all others depend on: `init` (creates `osmprj.toml`) and `add` (registers a data source).

The `osmprj.toml` file is project-local and drives every subsequent command. The Geofabrik index (`index-v1.json`) is the authoritative catalog of downloadable OSM extracts and must be fetched before `add` can validate user input.

## Goals / Non-Goals

**Goals:**
- Full `osmprj.toml` data model in Rust (`ProjectConfig`, `SourceConfig`, `TopicsConfig`)
- Schema name normalization: replace `/` and `-` with `_` in source names
- `geofabrik.rs`: fetch `index-v1.json` once, cache at OS user cache dir, expose ID lookup
- `init` command: write minimal `osmprj.toml` with optional `--db`; error if file exists
- `add` command: Geofabrik path (validate ID → append source block) and local file path (`--path --name`)
- Non-destructive TOML editing via `toml_edit` (preserves comments and formatting of existing file)

**Non-Goals:**
- Implementing `sync`, `remove`, or `destroy`
- Downloading actual `.osm.pbf` files (that is `sync`'s job)
- Themepark cache or Lua config generation (future change)
- Python implementation parity

## Decisions

### TOML mutation: `toml_edit` over `toml` + reserialize

`add` appends a new `[sources.<name>]` block to an existing `osmprj.toml`. Using the plain `toml` crate would require deserializing the whole document, mutating a Rust struct, and reserializing — which strips comments. `toml_edit` treats the file as a document with preserved formatting, making surgical appends clean.

**Alternatives considered:**
- *Append raw string*: fragile, breaks if the file has trailing content
- *toml + reserialize*: loses user comments

### HTTP client: `reqwest` blocking feature

The Geofabrik index fetch is a one-shot synchronous operation during `add`. Using `reqwest`'s blocking API keeps the code straightforward without requiring an async context in the command handler. The binary already has a tokio runtime for `db.rs`, but it isn't needed here.

**Alternatives considered:**
- *`ureq`*: lighter, no async story at all; fine choice but `reqwest` is more familiar and consistent with potential future async use
- *`curl` bindings*: heavy C dependency

### Cache location: `dirs::cache_dir()`

The `dirs` crate provides `cache_dir()` which maps to `~/.cache` on Linux, `~/Library/Caches` on macOS, and `%LOCALAPPDATA%` on Windows — the correct OS-conventional locations. Cache path: `<cache_dir>/osmprj/geofabrik-index-v1.json`.

Cache strategy: **fetch once, never invalidate**. The Geofabrik index is stable; users who need a refresh can delete the cache file manually. No TTL, no `--refresh` flag at this stage.

### Source disambiguation: `path` key presence

A `[sources.<name>]` block is a Geofabrik source if `path` is absent; it is a local file source if `path` is present. The source name is the Geofabrik ID in the first case, and an arbitrary label in the second.

### Schema name normalization

Applied to the source name when `schema` is not explicitly set: replace every `/` and `-` with `_`. This is a pure string transform with no ambiguity. Applied at config load time, not at write time, so the stored `osmprj.toml` always shows the normalized value when `schema` is explicit, and the default is computed on the fly when not.

### Module layout

```
src/
  main.rs             — CLI wiring only
  config.rs           — ProjectConfig, SourceConfig, TopicsConfig, load/save
  geofabrik.rs        — index fetch, cache, ID lookup
  commands/
    mod.rs
    init.rs
    add.rs
  db.rs               — unchanged placeholder
```

Commands are moved to a `commands/` submodule to keep `main.rs` thin and set the pattern for future commands.

## Risks / Trade-offs

- [`toml_edit` API surface is larger than `toml`] → Only a small subset is needed (table insertion); risk is low once the pattern is established.
- [Geofabrik index is ~500 KB JSON] → Single fetch, then disk. No memory pressure concern.
- [No cache invalidation] → If Geofabrik adds a new region, users need to delete the cache manually to see it. Acceptable given the index changes rarely and `add` is not a hot path.
- [`reqwest` adds significant compile time] → Acceptable for a CLI tool; can be revisited if build times become painful.
