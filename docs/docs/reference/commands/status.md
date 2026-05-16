---
id: status
title: status
sidebar_position: 3
---

# `osmprj status`

```
osmprj status
```

Shows the configured database connection and the state of each registered source. Requires `osmprj.toml` in the current directory.

## Example output

```
  database:  postgres://postgres@localhost/osm  ✓ connected

  source   schema   status
  ------   ------   ------
  germany  germany  ✓
  france   france   ✗  — run 'osmprj sync' to import
```

If no database URL is configured, sources are still listed but without connection status.
