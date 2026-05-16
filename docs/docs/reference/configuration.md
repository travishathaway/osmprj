---
id: configuration
title: Configuration Reference
sidebar_position: 2
---

# Configuration Reference

osmprj is configured via `osmprj.toml` in your project directory. The file is created by `osmprj init` and updated automatically by `osmprj add` and `osmprj remove`.

## `[project]` fields

| Field | Type | Default | Description |
|---|---|---|---|
| `database_url` | string | — | PostgreSQL connection URL. Required for `sync` and `status`. Avoid storing passwords here if the file is committed to version control — use `OSMPRJ_DATABASE_URL` instead. |
| `data_dir` | string | OS cache dir | Directory for downloaded PBF files. |
| `log_dir` | string | `./logs` | Directory for osm2pgsql log files. |
| `ssd` | bool | `true` | Set to `false` if `data_dir` is on spinning disk. Raises the flat-nodes threshold from 8 GB to 30 GB. |
| `max_diff_size_mb` | integer | — | Maximum diff size in MB for replication updates. |

## `[sources.<name>]` fields

| Field | Type | Description |
|---|---|---|
| `theme` | string | Theme name to use for this source. See [`osmprj themes list`](/docs/reference/commands/themes-list) for all available options. |
| `schema` | string | PostgreSQL schema name. Auto-derived from source name if omitted (e.g. `europe/france` → `europe_france`). |
| `path` | string | Path to a local PBF file. Use instead of a Geofabrik ID for local imports. |

## Full example

```toml
[project]
# PostgreSQL connection URL.
# WARNING: avoid storing passwords here if osmprj.toml is committed to version control.
# Use OSMPRJ_DATABASE_URL or a .env file instead (see Credential Resolution Order below).
database_url = "postgres://user@localhost/osm"

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
theme = "generic"
schema = "custom"
```

## Credential resolution order

osmprj resolves the database URL using the following priority order (highest first):

### 1. `OSMPRJ_DATABASE_URL` environment variable

Overrides everything. Ideal for CI and scripted environments. Never written to disk by osmprj.

```bash
export OSMPRJ_DATABASE_URL="postgres://user:pass@localhost/osm"
osmprj sync
```

### 2. `.env` file

osmprj loads a `.env` file next to `osmprj.toml` at startup, if one exists. Variables already set in the shell environment take priority. Add `.env` to your `.gitignore` to keep credentials out of version control.

```bash
# .env  (gitignored)
OSMPRJ_DATABASE_URL=postgres://user:pass@localhost/osm
```

:::note
All variables in `.env` are loaded into osmprj's environment, so `OSMPRJ_THEME_PATH`, `NO_COLOR`, and any other supported env vars work here too.
:::

### 3. `database_url` in `osmprj.toml`

The fallback. Safe to use when the URL contains no password (e.g. local trust auth) or in a private repository.

```toml
[project]
database_url = "postgres://postgres@localhost/osm"
```

For a narrative walkthrough of all methods, see the [Storing Credentials Securely](/docs/guides/storing-credentials) guide.
