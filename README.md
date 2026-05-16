
<p align="center">
  <img src="https://travishathaway.github.io/osmprj/img/osmprj-logo-big.png" width="25%">
</p>
<p align="center">
  <em>A friendly, modern tool for managing OpenStreetMap data with PostgreSQL</em>
</p>

<p align="center">
  <a href="https://github.com/travishathaway/osmprj/actions?query=workflow%3ACI" target="_blank">
      <img src="https://github.com/travishathaway/osmprj/workflows/CI/badge.svg" alt="Test">
  </a>
  <a href="https://travishathaway.github.io/osmprj" target="_blank">
      <img src="https://img.shields.io/static/v1?label=Documentation&message=View&color=blue&logo=readme&logoColor=white" alt="Documentation">
  </a>
  <a href="https://github.com/sponsors/travishathaway" target="_blank">
      <img src="https://img.shields.io/badge/Sponsor-GitHub-pink?logo=github" alt="Sponsor">
  </a>
</p>

A command-line tool for managing OpenStreetMap data imports into PostgreSQL that provides a project based workflow similar to tools like [uv](https://docs.astral.sh/uv/), [Cargo](https://crates.io) and [pixi](https://pixi.sh). It wraps [osm2pgsql](https://osm2pgsql.org) to automate downloading PBF files from [Geofabrik](https://downloads.geofabrik.de), running incremental updates, and offers 9 built-in themes you can use to customize the layout of your database.

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
  - [Credential Resolution Order](#credential-resolution-order)
- [Contributing](#contributing)
- [Sponsor This Project](#sponsor-this-project)

---

## Installation

The best way to install `osmprj` is with [pixi](https://pixi.sh).

With `pixi global`:

```
pixi global install -c gis-forge -c conda-forge osmprj
```

Or by creating a standalone [conda](https://docs.conda.io) environment:

```
conda create -n osmprj -c gis-forge -c conda-forge osmprj
```

---

## Quick Start

The typical workflow is: initialise a project, add one or more data sources, then sync.

```bash
# 1. Create a project file in the current directory
osmprj init

# 2. Add a Geofabrik region (uses shortbread theme)
osmprj add germany --theme shortbread

# 3. Check what will be synced
osmprj status

# 4. Download and import the data
osmprj sync
```

> [!WARNING]
> **Do not store database passwords in `osmprj.toml`.** The file is typically committed to version control, which would expose credentials. Instead, provide the database URL via the `OSMPRJ_DATABASE_URL` environment variable or the `database_url_command` config field. See the [Credential Resolution Order](#credential-resolution-order) section for details.

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

> [!TIP]
> If your connection URL contains a password, prefer `OSMPRJ_DATABASE_URL` (env var) or `database_url_command` instead of storing it in this file. See [Credential Resolution Order](#credential-resolution-order).

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
# PostgreSQL connection URL.
# WARNING: avoid storing passwords here if osmprj.toml is committed to version control.
# Use OSMPRJ_DATABASE_URL or database_url_command instead (see below).
database_url = "postgres://user@localhost/osm"

# Shell command whose stdout is used as the full database URL.
# Runs via `sh -c` on Unix or `cmd /C` on Windows.
# Takes precedence over database_url when set.
# OSMPRJ_DATABASE_URL environment variable takes precedence over both.
#
# Examples:
#   database_url_command = "pass show osmprj/db-url"
#   database_url_command = "op read op://Personal/osmprj-db/url"
#   database_url_command = "gpg --quiet --decrypt ~/.osmprj-db-url.gpg"
#   database_url_command = "aws secretsmanager get-secret-value --secret-id osmprj/db --query SecretString --output text"
# database_url_command = "pass show osmprj/db-url"

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

### Credential Resolution Order

osmprj resolves the database URL using the following priority order (highest first):

1. **`OSMPRJ_DATABASE_URL` environment variable** — overrides everything. Ideal for CI and scripted environments.

   ```bash
   export OSMPRJ_DATABASE_URL="postgres://user:pass@localhost/osm"
   osmprj sync
   ```

2. **`.env` file** in the project directory — osmprj loads a `.env` file next to `osmprj.toml` at startup, if one exists. Variables already set in the shell environment take priority. Add `.env` to your `.gitignore` to keep credentials out of version control.

   ```bash
   # .env  (gitignored)
   OSMPRJ_DATABASE_URL=postgres://user:pass@localhost/osm
   ```

   > **Note:** All variables in `.env` are loaded into osmprj's environment, so `OSMPRJ_THEME_PATH`, `NO_COLOR`, and any other supported env vars work here too.

3. **`database_url_command`** in `osmprj.toml` — osmprj runs the command via the system shell and reads the URL from stdout. Use this to integrate with your existing secret manager.

   ```toml
   [project]
   database_url_command = "pass show osmprj/db-url"
   ```

   Real-world examples:

   | Tool | Example |
   |---|---|
   | `pass` (Unix Password Manager) | `pass show osmprj/db-url` |
   | 1Password CLI | `op read op://Personal/osmprj-db/url` |
   | GPG-encrypted file | `gpg --quiet --decrypt ~/.osmprj-db-url.gpg` |
   | AWS Secrets Manager | `aws secretsmanager get-secret-value --secret-id osmprj/db --query SecretString --output text` |
   | HashiCorp Vault | `vault kv get -field=url secret/osmprj/db` |
   | Shell script | `/home/user/.config/osmprj/get-db-url.sh` |

4. **`database_url`** in `osmprj.toml` — the fallback. Safe to use when the URL contains no password (e.g. local trust auth) or in a private repository.

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
