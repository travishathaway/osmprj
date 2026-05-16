---
id: sync
title: sync
sidebar_position: 4
---

# `osmprj sync`

```
osmprj sync [SOURCE...] [-v]
```

Downloads and imports all registered sources, or only the named subset. On first run it performs a full import; on subsequent runs it applies incremental updates via `osm2pgsql-replication`.

## Options

- **`SOURCE...`** — One or more source names to sync. Defaults to all sources if omitted.
- **`-v` / `--verbose`** — Stream `osm2pgsql` log output to the terminal in addition to writing it to the log file.

## How it works

1. **Classify** — Checks the database to determine which sources have already been imported and have replication initialised (update mode) vs. those that need a fresh import.
2. **Download** — For each Geofabrik source that needs a fresh import and has not been downloaded yet, streams the PBF file to the OS cache directory with a progress bar. MD5 checksums are verified against Geofabrik's sidecar files. Already-downloaded files are skipped.
3. **Tune** — Automatically selects `osm2pgsql` flags based on your system RAM, PBF file size, and whether the storage is SSD:
   - Uses `--flat-nodes` for large files (≥ 8 GB on SSD, ≥ 30 GB on HDD).
   - Sets `--cache` to up to 66% of system RAM for smaller files.
4. **Import** — Runs `osm2pgsql --create --slim --output=flex` for each fresh source.
5. **Replication init** — Runs `osm2pgsql-replication init` immediately after each fresh import.
6. **Update** — For sources already in update mode, runs `osm2pgsql-replication update` to apply changes since the last sync.

Logs for each source are written to `./logs/<source-name>.log` (configurable via `log_dir`).

## Examples

```bash
# Sync everything
osmprj sync

# Sync a specific source
osmprj sync germany

# Sync with verbose osm2pgsql output
osmprj sync -v
```
