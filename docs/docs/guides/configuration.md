---
id: configuration
title: Configuration Reference
sidebar_position: 4
---

# Configuration Reference

osmprj is configured via `osmprj.toml` in your project directory. The file is created by `osmprj init` and updated by `osmprj add` / `osmprj remove`.

## `[project]` fields

| Field | Type | Default | Description |
|---|---|---|---|
| `database_url` | string | — | PostgreSQL connection URL. Required for `sync` and `status`. |
| `data_dir` | string | OS cache dir | Directory for downloaded PBF files. |
| `log_dir` | string | `./logs` | Directory for osm2pgsql log files. |
| `ssd` | bool | `true` | Set to `false` if `data_dir` is on spinning disk. Raises the flat-nodes threshold from 8 GB to 30 GB. |
| `max_diff_size_mb` | integer | — | Maximum diff size in MB for replication updates. |

## `[sources.<name>]` fields

| Field | Type | Description |
|---|---|---|
| `theme` | string | Theme name to use for this source. |
| `schema` | string | PostgreSQL schema name. Auto-derived from source name if omitted. |
| `path` | string | Path to a local PBF file (use instead of a Geofabrik ID). |

## Example

```toml
[project]
database_url = "postgres://user:pass@localhost/osm"
data_dir = "/mnt/data/osm"
ssd = true

[sources.germany]
theme = "shortbread"
schema = "germany"

[sources."europe/france"]
theme = "shortbread-gen"
schema = "france"
```

<!-- TODO: expand with postprocess block, full field descriptions, environment variable overrides -->
