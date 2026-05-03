# osmprj

A command-line tool for managing OpenStreetMap data imports into PostgreSQL. It wraps `osm2pgsql` to automate downloading PBF files from Geofabrik, tuning import parameters for your hardware, running incremental updates, and tracking source state across runs.

> [!WARNING]
> **osmprj is experimental software under active development.** Commands, configuration formats, and behavior may change without notice between versions. It is not yet recommended for production use. Feedback and bug reports are very welcome — see the [Contributing](#contributing) section below.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
  - [init](#init)
  - [add](#add)
  - [status](#status)
  - [sync](#sync)
  - [remove](#remove)
  - [themes list](#themes-list)
- [Configuration Reference](#configuration-reference)
- [Contributing](#contributing)
- [Sponsor This Project](#sponsor-this-project)

---

## Installation

The best way to install `osmprj` is as a conda package.

With `pixi global`:

```
pixi global install -c gis-forge -c conda-forge osmprj
```

Or by creating a standalone conda environment:

```
conda create -n osmprj -c gis-forge -c conda-forge osmprj
```

---

## Quick Start

The typical workflow is: initialise a project, add one or more data sources, then sync.

```bash
# 1. Create a project file in the current directory
osmprj init --db "postgres://user:pass@localhost/osm"

# 2. Add a Geofabrik region (uses shortbread theme)
osmprj add germany --theme shortbread

# 3. Check what will be synced
osmprj status

# 4. Download and import the data
osmprj sync
```

On the first run, `sync` downloads the PBF from Geofabrik, auto-tunes the `osm2pgsql` parameters for your system, and initialises replication. On subsequent runs it applies only the changes since the last update.

---

## Command Reference

### init

```
osmprj init [--db <DATABASE_URL>]
```

Creates an `osmprj.toml` project file in the current directory. Fails if one already exists.

- `--db` — Writes the database connection URL into `[project].database_url`. If omitted, a commented-out placeholder is added instead.

**Example**

```bash
osmprj init --db "postgres://postgres@localhost/osm"
```

The resulting `osmprj.toml`:

```toml
[project]
database_url = "postgres://postgres@localhost/osm"

# Add sources with: osmprj add <geofabrik-id> --theme <theme>
```

---

### add

```
osmprj add <GEOFABRIK_ID>... [--theme <THEME>] [--schema <SCHEMA>]
osmprj add --path <FILE> --name <NAME> [--theme <THEME>] [--schema <SCHEMA>]
```

Registers one or more data sources in `osmprj.toml`. If a database URL is configured, `add` also creates the PostgreSQL schema immediately.

**Adding Geofabrik regions**

Pass one or more region IDs (the path component from [download.geofabrik.de](https://download.geofabrik.de)). The index is fetched and cached automatically.

```bash
# Single region
osmprj add germany --theme shortbread

# Multiple regions at once (schema names are auto-derived)
osmprj add europe/france europe/spain --theme shortbread

# Override the schema name (only valid for a single ID)
osmprj add europe/france --theme shortbread --schema france
```

**Adding a local PBF file**

Use `--path` and `--name` together. The file is used directly without any download.

```bash
osmprj add --path /data/my-region.osm.pbf --name my-region --theme shortbread
```

See `osmprj themes list` for a full list of available themes.

Schema names are auto-derived from the source name by replacing `/` and `-` with `_` (e.g. `europe/france` → `europe_france`). Use `--schema` to override.

---

### status

```
osmprj status
```

Shows the configured database connection and the state of each registered source. Requires `osmprj.toml` in the current directory.

**Example output**

```
  database:  postgres://postgres@localhost/osm  ✓ connected

  source   schema   status
  ------   ------   ------
  germany  germany  ✓
  france   france   ✗  — run 'osmprj sync' to import
```

If no database URL is configured, sources are still listed but without connection status.

---

### sync

```
osmprj sync [SOURCE...] [-v]
```

Downloads and imports all registered sources, or only the named subset. On first run it performs a full import; on subsequent runs it applies incremental updates via `osm2pgsql-replication`.

- **Source selection** — Pass one or more source names to sync only those. Defaults to all sources.
- `-v` / `--verbose` — Stream `osm2pgsql` log output to the terminal in addition to writing it to the log file.

**How it works**

1. **Classify** — Checks the database to determine which sources have already been imported and have replication initialised (update mode) vs. those that need a fresh import.
2. **Download** — For each Geofabrik source that needs a fresh import and has not been downloaded yet, streams the PBF file to the OS cache directory with a progress bar. MD5 checksums are verified against Geofabrik's sidecar files. Already-downloaded files are skipped.
3. **Tune** — Automatically selects `osm2pgsql` flags based on your system RAM, PBF file size, and whether the storage is SSD:
   - Uses `--flat-nodes` for large files (≥ 8 GB on SSD, ≥ 30 GB on HDD).
   - Sets `--cache` to up to 66% of system RAM for smaller files.
4. **Import** — Runs `osm2pgsql --create --slim --output=flex` for each fresh source.
5. **Replication init** — Runs `osm2pgsql-replication init` immediately after each fresh import.
6. **Update** — For sources already in update mode, runs `osm2pgsql-replication update` to apply changes since the last sync.

Logs for each source are written to `./logs/<source-name>.log` (configurable via `log_dir`).

**Examples**

```bash
# Sync everything
osmprj sync

# Sync a specific source
osmprj sync germany

# Sync with verbose osm2pgsql output
osmprj sync -v
```

---

### remove

```
osmprj remove <SOURCE>... [--dry-run] [-f | --force]
```

Removes one or more data sources from the project. This command:

1. Removes the source entry from `osmprj.toml`
2. Drops the corresponding PostgreSQL schema and all its data (`DROP SCHEMA … CASCADE`)
3. Removes the source entry from `osmprj.lock`

Before making any changes, osmprj prints a summary of what will be removed and prompts for confirmation. This is a destructive operation — dropped schemas cannot be recovered without a database backup.

**Flags**

- `--dry-run` — Print what would be removed without making any changes. No confirmation prompt is shown.
- `-f` / `--force` — Skip the confirmation prompt. Useful in scripts or CI environments.

**Examples**

```bash
# Preview what would be removed
osmprj remove germany --dry-run

# Remove a single source (prompts for confirmation)
osmprj remove germany

# Remove multiple sources at once
osmprj remove europe/france europe/spain

# Remove without prompting (e.g. in a script)
osmprj remove germany --force
osmprj remove germany -f
```

**What happens to the PBF file?**

The downloaded `.osm.pbf` file in the cache directory is **not** deleted. It may be large and reusable if you re-add the source later. Delete it manually from the directory shown in `data_dir` if you no longer need it.

### themes list

```
osmprj themes list
```

List all available themes.

Currently available themes:

- **generic**, generic basic topics theme
- **nwr**, Node/Way/Relation (NWR) topic theme
- **osmcarto**, OSM Carto theme
- **pgosm**, PgOSM Flex theme: default variation
- **pgosm-basic** PgOSM Flex theme: basic variation
- **pgosm-everything**, PgOSM Flex theme: everything variation
- **pgosm-minimal**, PgOSM Flex theme: minimal variation
- **shortbread**, shortbread theme
- **shortbread-gen**, shortbread theme with generalization

If you want to expand and add your own themes, append to the `OSMPRJ_THEME_PATH` environment variable so that your themes are discoverable:

```
export OSMPRJ_THEME_PATH="$OSMPRJ_THEME_PATH:/your/themes"
```

---

## Configuration Reference

`osmprj.toml` lives in your project directory. All fields under `[project]` are optional.

```toml
[project]
# PostgreSQL connection URL (required for sync and status)
database_url = "postgres://user:pass@localhost/osm"

# Directory for downloaded PBF files.
# Default: <OS cache dir>/osmprj/geofabrik/
data_dir = "/mnt/data/osm"

# Directory for osm2pgsql log files. Default: ./logs
log_dir = "/var/log/osmprj"

# Set to false if data_dir is on spinning disk.
# Raises the flat-nodes threshold from 8 GB to 30 GB. Default: true
ssd = true

# Maximum diff size in MB for replication updates. Optional.
max_diff_size_mb = 500

# Sources are added by `osmprj add` but can also be edited by hand.
[sources.germany]
theme = "shortbread"
schema = "germany"

[sources."europe/france"]
theme = "shortbread-gen"
schema = "france"

[sources.local-extract]
path = "/data/custom.osm.pbf"
theme = "basic"
schema = "custom"
```

---

## Contributing

Contributions are welcome. osmprj is in early development so there is plenty of room to help.

### Reporting Bugs

Please [open an issue](https://github.com/travishathaway/osmprj/issues/new) and include:

- Your operating system and architecture
- The output of `osmprj --version`
- The `osmprj.toml` you were using (redact credentials)
- The full error message or unexpected output
- The relevant section of the log file if the failure was in `sync`

### Submitting Pull Requests

1. Fork the repository and create a branch from `main`.
2. Make your changes. Run `cargo fmt` and `cargo clippy` before committing.
3. Add or update tests where appropriate (`cargo test`).
4. Open a pull request with a clear description of what the change does and why.

For non-trivial changes, opening an issue first to discuss the approach is encouraged.

### Development Setup

```bash
git clone https://github.com/travishathaway/osmprj
cd osmprj

# Start a pixi shell
pixi shell -e dev

# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

Integration tests require a running PostgreSQL instance with PostGIS. See `tests/integration/conftest.py` for setup details.

---

## Sponsor This Project

osmprj is developed and maintained as free, open-source software. If you find it useful and would like to support continued development, please consider sponsoring via GitHub Sponsors:

**[github.com/sponsors/travishathaway](https://github.com/sponsors/travishathaway)**

Sponsorships help fund time for new features, bug fixes, documentation, and keeping the project maintained. Every contribution, large or small, is appreciated.
