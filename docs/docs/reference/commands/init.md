---
id: init
title: init
sidebar_position: 1
---

# `osmprj init`

```
osmprj init [--db <DATABASE_URL>]
```

Creates an `osmprj.toml` project file in the current directory. Fails if one already exists.

## Options

- `--db` — Writes the database connection URL into `[project].database_url`. If omitted, a commented-out placeholder is added instead.

## Example

```bash
osmprj init --db "postgres://postgres@localhost/osm"
```

The resulting `osmprj.toml`:

```toml
[project]
database_url = "postgres://postgres@localhost/osm"

# Add sources with: osmprj add <geofabrik-id> --theme <theme>
```

:::tip
If your connection URL contains a password, prefer `OSMPRJ_DATABASE_URL` (env var) or a `.env` file instead of storing it in this file. See [Storing Credentials Securely](/docs/guides/storing-credentials).
:::
