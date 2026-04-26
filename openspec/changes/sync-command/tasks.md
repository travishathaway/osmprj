## 1. Dependencies and Config

- [ ] 1.1 Add `sysinfo`, `futures`, `md-5`, `chrono`, `tempfile`, `flate2`, `tar` to `Cargo.toml`
- [ ] 1.2 Add `data_dir: Option<String>`, `log_dir: Option<String>`, `ssd: Option<bool>` to `ProjectSettings` in `src/config.rs`
- [ ] 1.3 Implement `ProjectSettings::effective_data_dir()` → `PathBuf` (config value or `<cache_dir>/osmprj/geofabrik/`)
- [ ] 1.4 Implement `ProjectSettings::effective_log_dir()` → `PathBuf` (config value or `./logs`)
- [ ] 1.5 Add `verbose: bool` global flag to `Cli` in `src/main.rs` with `#[arg(short = 'v', long, global = true)]`
- [ ] 1.6 Add `sources: Vec<String>` positional argument to the `Sync` variant in `src/main.rs`

## 2. Lock File (`src/lock.rs`)

- [ ] 2.1 Define `SourceLockEntry { url, md5, downloaded_at: DateTime<Utc> }` and `ThemeparkLockEntry { cached_at: DateTime<Utc> }`
- [ ] 2.2 Define `LockFile { sources: HashMap<String, SourceLockEntry>, themepark: Option<ThemeparkLockEntry> }`
- [ ] 2.3 Implement `LockFile::load()` — reads `osmprj.lock` from CWD; returns empty `LockFile` if absent
- [ ] 2.4 Implement `LockFile::save()` — writes to `osmprj.lock` using `toml_edit` (preserves comments)
- [ ] 2.5 Implement `LockFile::set_source()` — upserts a `SourceLockEntry` and immediately calls `save()`
- [ ] 2.6 Add `OsmprjError` variants: `LockReadFailed`, `LockWriteFailed`

## 3. Tuner (`src/tuner.rs`)

- [ ] 3.1 Define `TunerInput { system_ram_gb: f64, pbf_size_gb: f64, ssd: bool, database_url: String, effective_schema: String, pbf_path: PathBuf, style_path: PathBuf, data_dir: PathBuf }`
- [ ] 3.2 Implement `use_flat_nodes(input) -> bool` — threshold logic from design D4
- [ ] 3.3 Implement `get_cache_mb(input) -> u32` — slim_cache vs cache_max calculation
- [ ] 3.4 Implement `build_command(input) -> Vec<String>` — assembles full `osm2pgsql` argv with all flags
- [ ] 3.5 Implement `system_ram_gb() -> f64` — reads from `sysinfo::System::total_memory()`
- [ ] 3.6 Write unit tests covering: small file (no flat-nodes), 10 GB SSD (flat-nodes), 35 GB non-SSD (flat-nodes), cache capped by RAM, command always has `--slim --create` and no `--drop`

## 4. Themepark Cache (`src/themepark.rs`)

- [ ] 4.1 Implement `ensure_cached(cache_dir: &Path, lock: &mut LockFile) -> Result<PathBuf, OsmprjError>` — downloads and extracts tarball if not present; updates lock
- [ ] 4.2 Implement tarball download using async `reqwest::Client` with a simple spinner (no byte total available)
- [ ] 4.3 Implement tarball extraction using `flate2::read::GzDecoder` + `tar::Archive::unpack()`
- [ ] 4.4 Implement `resolve_config_file(themepark_dir: &Path, theme: &str) -> Result<PathBuf, OsmprjError>` — hardcoded mapping table with directory-listing fallback
- [ ] 4.5 Implement `generate_lua_tempfile(themepark_dir: &Path, base_config: &Path, topics_config: &TopicsConfig) -> Result<NamedTempFile, OsmprjError>` — reads base topics, applies add/remove/list, writes tempfile with `package.path` header
- [ ] 4.6 Add `OsmprjError` variants: `ThemeparkDownloadFailed`, `ThemeparkExtractFailed`, `ThemeNotFound { theme: String }`

## 5. Download Phase (`src/commands/sync.rs` — Phase 1)

- [ ] 5.1 Create `src/commands/sync.rs` with `pub async fn run(sources: Vec<String>, verbose: bool, config: &ProjectConfig) -> Result<(), OsmprjError>`
- [ ] 5.2 Validate source filter: if `sources` is non-empty, check all names exist in `config.sources`; error early if any are unknown
- [ ] 5.3 Check `osm2pgsql` and `osm2pgsql-replication` are on `PATH` before any downloads begin; emit clear error if not
- [ ] 5.4 Resolve effective `data_dir`; create directory if absent
- [ ] 5.5 Build a `MultiProgress` and spawn one `tokio::JoinSet` task per Geofabrik source (skip sources with `path` set)
- [ ] 5.6 Per download task: check lock for existing entry → skip if present; otherwise stream download with `reqwest::Client`, update `ProgressBar` per chunk
- [ ] 5.7 Per download task: fetch `<url>.md5`, verify hash using `md-5` crate; on mismatch return `Err`
- [ ] 5.8 Per download task: on success call `lock.set_source()` (incremental write) and `bar.finish_with_message("✓")`
- [ ] 5.9 Drain `JoinSet`; collect all errors; if any errors: print summary and return `Err` without proceeding to import

## 6. Import Phase (`src/commands/sync.rs` — Phase 2)

- [ ] 6.1 Call `MultiProgress::clear()` then print transition line: `🗺  All N files ready — starting imports`
- [ ] 6.2 Ensure themepark is cached via `themepark::ensure_cached()`
- [ ] 6.3 For each source (in osmprj.toml order, filtered by source list):
        resolve PBF path (data_dir or config.path), get file size in GB
- [ ] 6.4 Call `tuner::build_command()` with system RAM, PBF size, ssd flag, db URL, schema, PBF path, and style path
- [ ] 6.5 Resolve style path: call `themepark::resolve_config_file()` or `generate_lua_tempfile()` if topics are set
- [ ] 6.6 Create `log_dir` if absent; open log file at `log_dir/<source_name>.log`
- [ ] 6.7 Spawn `tokio::process::Command` for osm2pgsql with `stdout(Stdio::piped())` and `stderr(Stdio::piped())`
- [ ] 6.8 Start globe spinner `ProgressBar` with tick strings `["🌍 ", "🌎 ", "🌏 ", "🌐 "]` at 250 ms
- [ ] 6.9 Concurrently pipe stdout and stderr to log file (and to terminal if `verbose`)
- [ ] 6.10 On non-zero exit: stop spinner, log error, return `Err` immediately (do not process remaining sources)
- [ ] 6.11 On success: stop spinner with `finish_with_message("✓ <source> imported")`

## 7. Replication Init

- [ ] 7.1 After all imports succeed, for each imported source run `osm2pgsql-replication init -d <database_url> --schema <effective_schema>` via `tokio::process::Command`
- [ ] 7.2 Capture output to `log_dir/<source_name>-replication-init.log`; stream to terminal if `verbose`
- [ ] 7.3 On failure: report error and exit non-zero; do not run replication init for remaining sources
- [ ] 7.4 On all success: print `🌐  Sync complete. Replication enabled for N source(s).`

## 8. Wire Up and Integration

- [ ] 8.1 In `src/commands/mod.rs`, add `pub mod sync`
- [ ] 8.2 In `src/main.rs`, match `Commands::Sync { sources, verbose }` and call `commands::sync::run(sources, verbose, &config).await`
- [ ] 8.3 Pass `verbose` from global `Cli` field into sync run (propagate the flag correctly)

## 9. Integration Tests

- [ ] 9.1 Add integration test: `sync` with no project file exits non-zero
- [ ] 9.2 Add integration test: `sync` with unknown source name exits non-zero before any output
- [ ] 9.3 Add integration test for tuner unit: small file produces correct cache MB and no flat-nodes flag
- [ ] 9.4 Add integration test for tuner unit: large file (≥8 GB) produces `--flat-nodes` and `--cache=0`
- [ ] 9.5 Add integration test: lock file written after download (mock or fixture PBF + MD5 server, or skip-if-no-network)
- [ ] 9.6 Add integration test: second sync with existing lock entry skips download
