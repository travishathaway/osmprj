
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

The typical workflow is: initialize a project, add one or more data sources, then sync.

```bash
# 1. Create a project file in the current directory
osmprj init --db postgresql://user@localhost:5432/db

# 2. Add a Geofabrik region (uses shortbread theme)
osmprj add germany --theme shortbread

# 3. Check what will be synced
osmprj status

# 4. Download and import the data
osmprj sync
```

> [!WARNING]
> **Avoid storing database passwords in `osmprj.toml`.** Instead, we recommend using the [.pgpass](https://www.postgresql.org/docs/current/libpq-pgpass.html) file. See the [Storing Credentials Securely](https://osmprj.dev/docs/guides/storing-credentials) guide for more details.

On the first run, `sync` downloads the PBF from Geofabrik, auto-tunes the `osm2pgsql` parameters for your system, and initializes replication. On subsequent runs it applies only the changes since the last update.

---

## Commands

| Command | Description |
|---|---|
| `osmprj init` | Create a new `osmprj.toml` project file |
| `osmprj add` | Register a data source (Geofabrik region or local PBF) |
| `osmprj status` | Show database connection and source states |
| `osmprj sync` | Download and import/update all sources |
| `osmprj remove` | Remove a source from the project and database |
| `osmprj themes list` | List all available themes |

For full usage details, flags, and examples, see the [Command Reference](https://osmprj.dev/docs/reference/commands) on the docs site.

---

## Configuration

osmprj is configured via `osmprj.toml` in your project directory. Key fields:

- `database_url` — PostgreSQL connection URL
- `data_dir` — directory for downloaded PBF files (default: OS cache dir)
- `log_dir` — directory for log files (default: `./logs`)
- `ssd` — set to `false` if `data_dir` is on spinning disk (default: `true`)
- `max_diff_size_mb` — maximum replication diff size in MB

> **Credential resolution order:** `OSMPRJ_DATABASE_URL` env var → `.env` file → `database_url` in `osmprj.toml`

For the full configuration reference, see [Configuration](https://osmprj.dev/docs/reference/configuration) on the docs site.

---

## Contributing

Contributions are welcome. This project is still early in development so there is plenty of room to help.

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
