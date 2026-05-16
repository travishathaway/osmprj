---
id: add
title: add
sidebar_position: 2
---

# `osmprj add`

```
osmprj add <GEOFABRIK_ID>... [--theme <THEME>] [--schema <SCHEMA>]
osmprj add --path <FILE> --name <NAME> [--theme <THEME>] [--schema <SCHEMA>]
```

Registers one or more data sources in `osmprj.toml`. If a database URL is configured, `add` also creates the PostgreSQL schema immediately.

## Adding Geofabrik regions

Pass one or more region IDs (the path component from [download.geofabrik.de](https://download.geofabrik.de)). The index is fetched and cached automatically.

```bash
# Single region
osmprj add germany --theme shortbread

# Multiple regions at once (schema names are auto-derived)
osmprj add europe/france europe/spain --theme shortbread

# Override the schema name (only valid for a single ID)
osmprj add europe/france --theme shortbread --schema france
```

## Adding a local PBF file

Use `--path` and `--name` together. The file is used directly without any download.

```bash
osmprj add --path /data/my-region.osm.pbf --name my-region --theme shortbread
```

## Schema naming

Schema names are auto-derived from the source name by replacing `/` and `-` with `_` (e.g. `europe/france` → `europe_france`). Use `--schema` to override.

See [`osmprj themes list`](/docs/reference/commands/themes-list) for all available themes.
