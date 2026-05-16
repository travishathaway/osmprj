---
id: remove
title: remove
sidebar_position: 5
---

# `osmprj remove`

```
osmprj remove <SOURCE>... [--dry-run] [-f | --force]
```

Removes one or more data sources from the project. This command:

1. Removes the source entry from `osmprj.toml`
2. Drops the corresponding PostgreSQL schema and all its data (`DROP SCHEMA … CASCADE`)
3. Removes the source entry from `osmprj.lock`

Before making any changes, osmprj prints a summary of what will be removed and prompts for confirmation. This is a destructive operation — dropped schemas cannot be recovered without a database backup.

## Options

- `--dry-run` — Print what would be removed without making any changes. No confirmation prompt is shown.
- `-f` / `--force` — Skip the confirmation prompt. Useful in scripts or CI environments.

## Examples

```bash
# Preview what would be removed
osmprj remove germany --dry-run

# Remove a single source (prompts for confirmation)
osmprj remove germany

# Remove multiple sources at once
osmprj remove europe/france europe/spain

# Remove without prompting (e.g. in a script)
osmprj remove germany --force
osmprj remove germany -f
```

## What happens to the PBF file?

The downloaded `.osm.pbf` file in the cache directory is **not** deleted. It may be large and reusable if you re-add the source later. Delete it manually from the directory shown in `data_dir` if you no longer need it.
