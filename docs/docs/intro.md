---
id: intro
title: Introduction
sidebar_position: 1
---

# Introduction

**osmprj** is a command-line tool for managing [OpenStreetMap](https://www.openstreetmap.org/) data imports into PostgreSQL. It wraps three underlying tools into a single, straightforward workflow:

- **[osm2pgsql](https://osm2pgsql.org/)** — imports OSM PBF files into a PostgreSQL/PostGIS database using a configurable Lua flex style
- **[Geofabrik](https://download.geofabrik.de/)** — provides regional OSM extracts that osmprj can download and keep up to date automatically
- **[osm2pgsql-replication](https://osm2pgsql.org/doc/manual.html#updating-an-existing-database)** — handles incremental updates so subsequent syncs apply only the changes since the last run

Instead of juggling these tools manually, osmprj provides a project file (`osmprj.toml`) that tracks your sources, and a small set of commands that orchestrate the full lifecycle: download, import, tune, and update.

## Status

:::warning
osmprj is experimental software under active development. Commands, configuration formats, and behaviour may change without notice between versions. It is not yet recommended for production use.
:::

Feedback and bug reports are welcome — [open an issue on GitHub](https://github.com/travishathaway/osmprj/issues/new).

## Next steps

→ [Getting Started](./getting-started.md) — install osmprj and run your first import in minutes.
