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
| `database_url` | string | — | PostgreSQL connection URL. Required for `sync` and `status`. Avoid storing passwords here if the file is committed to version control — use `database_url_command` or `OSMPRJ_DATABASE_URL` instead. |
| `database_url_command` | string | — | Shell command whose stdout is used as the full database URL. Runs via `sh -c` on Unix or `cmd /C` on Windows. Takes precedence over `database_url` when set. |
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
# Use OSMPRJ_DATABASE_URL or database_url_command instead (see below).
database_url = "postgres://user@localhost/osm"

# Shell command whose stdout is used as the full database URL.
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

### 3. `database_url_command` in `osmprj.toml`

osmprj runs the command via the system shell and reads the URL from stdout. Use this to integrate with your existing secret manager.

```toml
[project]
database_url_command = "pass show osmprj/db-url"
```

| Tool | Example |
|---|---|
| `pass` (Unix Password Manager) | `pass show osmprj/db-url` |
| 1Password CLI | `op read op://Personal/osmprj-db/url` |
| GPG-encrypted file | `gpg --quiet --decrypt ~/.osmprj-db-url.gpg` |
| AWS Secrets Manager | `aws secretsmanager get-secret-value --secret-id osmprj/db --query SecretString --output text` |
| HashiCorp Vault | `vault kv get -field=url secret/osmprj/db` |
| Shell script | `/home/user/.config/osmprj/get-db-url.sh` |

:::tip
In headless/CI environments, commands must be non-interactive (no passphrase prompts). Use the `OSMPRJ_DATABASE_URL` env var or `.env` file instead if your secret manager requires user interaction.
:::

### 4. `database_url` in `osmprj.toml`

The fallback. Safe to use when the URL contains no password (e.g. local trust auth) or in a private repository.

```toml
[project]
database_url = "postgres://postgres@localhost/osm"
```

For a narrative walkthrough of all four methods, see the [Storing Credentials Securely](/docs/guides/storing-credentials) guide.
