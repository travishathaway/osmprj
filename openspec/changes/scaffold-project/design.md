## Context

`osmprj` is a new Python CLI tool for managing OpenStreetMap data imports into PostgreSQL (via `osm2pgsql` and PostGIS). The `pyproject.toml` and `pixi.toml` declare all non-Python and Python dependencies respectively, but no source files exist yet. This design covers the initial package scaffold: directory layout, CLI wiring via `cyclopts`, configuration loading, and database connection factory.

## Goals / Non-Goals

**Goals:**
- Establish the `osmprj/` package with a working `osmprj` console script entry point
- Wire five placeholder sub-commands (`init`, `add`, `sync`, `remove`, `destroy`) via `cyclopts`
- Provide a `Settings` object (pydantic-settings) reading from env vars and an `osmprj.toml` project file
- Provide a `psycopg` connection context-manager consuming `Settings`
- Create `tests/unit/` and `tests/integration/` skeleton directories

**Non-Goals:**
- Implement actual OSM import logic or osm2pgsql wrappers
- Database migrations or schema management
- Multiple database backend support

## Decisions

### CLI: cyclopts

The proposal specifies `cyclopts` as the argument parsing library. It provides decorator-based command registration with automatic `--help` generation and is well-suited for a tool with a fixed set of sub-commands. Each command is a plain Python function decorated with `@app.command`.

**Entry point change:** `pyproject.toml` currently declares `pgosm-flex = "osmprj.main:main"`. This will be updated to `osmprj = "osmprj.main:app"` (or `main` wrapper) to match the proposal.

**Dependency:** `cyclopts` must be added to `project.dependencies` in `pyproject.toml` (Python dep) and to `tool.pixi.dependencies` if a conda-forge package exists; otherwise PyPI via `tool.pixi.pypi-dependencies`.

### Configuration: pydantic-settings with env vars + project TOML

`pydantic-settings` (already declared) reads from environment variables. A `Settings` class will also accept an `osmprj.toml` file in the current working directory as a layered source (CWD file → env vars → defaults). This lets the `init` command create a project config file that all other commands pick up automatically.

**Alternatives considered:**
- *Pure env vars*: cumbersome for repeated local invocations
- *Global user config*: less predictable; project-local config is standard for this class of tool

### Database: psycopg3 context-manager, no pool

`psycopg` v3 is already declared. A `connect()` context manager in `osmprj/db.py` reads connection params from `Settings` and yields a `psycopg.Connection`. No connection pool at this stage — the CLI is single-threaded and short-lived.

## Risks / Trade-offs

- [`cyclopts` availability in conda-forge] → Check before adding; fall back to `tool.pixi.pypi-dependencies` if not packaged for conda.
- [psycopg3 autocommit default is False] → Import commands will need explicit commits or `autocommit=True`; document in `db.py`.
- [osmprj.toml in CWD] → Commands run outside a project directory will skip the file gracefully; settings that require a DB connection will fail with a clear error at runtime.
