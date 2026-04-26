## Why

The `osmprj` package has its metadata and tooling configured (`pyproject.toml`) but contains no source code. The project needs an initial scaffold so development can begin: a working CLI entry point, configuration management, and a database connection layer that downstream features can build on. Python dependencies will be listed in project.dependencies and all dependencies (both python and non-python) dependencies will be listed in the tool.pixi.dependencies section.

## What Changes

- Add the `osmprj/` Python package directory with `__init__.py` and `main.py` (CLI entry point)
- Add a configuration module (`osmprj/config.py`) that reads settings from environment variables and a TOML config file using `pydantic-settings`
- Add a database module (`osmprj/db.py`) that wraps `psycopg` with a simple connection factory
- Add a `tests/` directory structure with `tests/unit/` and `tests/integration/` sub-packages
- Wire up the `osmprj` console script so `pixi run osmprj --help` works
- Use `cyclopts` as the argument parsing library
- Add placeholders for the following commands:
    - `init`: creates a new project by creating a `osmprj.toml` file in the current directory
    - `add`: adds a new data source to the project (e.g. a new .osm.pbf file location to the `osmprj.toml` file)
    - `sync`: syncs the OSM data files listed in `osmprj.toml` to the configured database
    - `remove`: removes a data source from `osmproj.toml`
    - `destroy`: removes all data from configured database

## Capabilities

### New Capabilities

- `cli`: Top-level CLI entry point (`main.py`) providing the `pgosm-flex` command with `--help` and version output
- `config`: Configuration loading from env vars and TOML file via `pydantic-settings`; exposes a single `Settings` object consumed by the rest of the app
- `db`: `psycopg`-based connection factory; provides a context-manager helper for acquiring a database connection using settings from `config`

### Modified Capabilities

## Impact

- Creates `osmprj/` package (new files, no existing code touched)
- Creates `tests/unit/` and `tests/integration/` skeleton directories
- No external API changes; no breaking changes
- All runtime dependencies (`psycopg`, `pydantic-settings`, `tomli`) are already declared in `pyproject.toml` as tool.pixi.dependencies; add the missing python dependencies to project.dependencies
