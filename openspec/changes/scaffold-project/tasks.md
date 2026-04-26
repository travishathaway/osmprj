## 1. Dependencies and Package Metadata

- [x] 1.1 Add `cyclopts` to `project.dependencies` in `pyproject.toml`
- [x] 1.2 Add `cyclopts` to `tool.pixi.pypi-dependencies` in `pyproject.toml` (conda-forge package unavailable)
- [x] 1.3 Update `project.scripts` in `pyproject.toml`: replace `pgosm-flex` entry with `osmprj = "osmprj.main:main"`

## 2. Package Skeleton

- [x] 2.1 Create `osmprj/__init__.py` (empty, marks package)
- [x] 2.2 Create `tests/__init__.py`, `tests/unit/__init__.py`, `tests/integration/__init__.py`

## 3. Configuration Module

- [x] 3.1 Create `osmprj/config.py` with a `Settings` class using `pydantic-settings` that reads env vars prefixed with `OSMPRJ_`
- [x] 3.2 Add `database_url: str | None = None` field to `Settings`
- [x] 3.3 Add a TOML file source to `Settings` that reads `osmprj.toml` from CWD if it exists, with env vars taking precedence

## 4. Database Module

- [x] 4.1 Create `osmprj/db.py` with a `connect()` context manager that accepts a `Settings` instance and yields a `psycopg.Connection`
- [x] 4.2 Add guard in `connect()` that raises `RuntimeError` with a clear message when `settings.database_url` is falsy

## 5. CLI Entry Point

- [x] 5.1 Create `osmprj/main.py` with a `cyclopts` `App` instance and a `main()` entry-point function
- [x] 5.2 Register `init` command placeholder (prints "not yet implemented")
- [x] 5.3 Register `add` command placeholder
- [x] 5.4 Register `sync` command placeholder
- [x] 5.5 Register `remove` command placeholder
- [x] 5.6 Register `destroy` command placeholder

## 6. Smoke Test

- [x] 6.1 Run `pixi install` to confirm cyclopts resolves without conflict
- [x] 6.2 Run `pixi run osmprj --help` and verify all five sub-commands appear in output
