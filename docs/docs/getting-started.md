---
id: getting-started
title: Getting Started
sidebar_position: 2
---

# Getting Started

## Installation

The recommended way to install osmprj is as a conda package.

**With `pixi global`** (recommended):

```bash
pixi global install -c gis-forge -c conda-forge osmprj
```

**With conda**:

```bash
conda create -n osmprj -c gis-forge -c conda-forge osmprj
conda activate osmprj
```

After installation, verify it works:

```bash
osmprj --version
```

## Prerequisites

osmprj requires a running PostgreSQL instance with the [PostGIS](https://postgis.net/) extension installed. It also requires `osm2pgsql` and `osm2pgsql-replication` to be available on your `PATH` — these are installed automatically when you install osmprj via conda.

## Quick Start

The typical workflow is four commands:

```bash
# 1. Create a project file in the current directory
osmprj init --db "postgres://user:pass@localhost/osm"

# 2. Add a Geofabrik region
osmprj add germany --theme shortbread

# 3. Check what will be synced
osmprj status

# 4. Download and import the data
osmprj sync
```

### What happens on first run

When you run `osmprj sync` for the first time:

1. The PBF file is downloaded from Geofabrik with a progress bar (MD5-verified)
2. osmprj tunes the `osm2pgsql` flags for your hardware (RAM, SSD/HDD, file size)
3. `osm2pgsql` performs a full import into your database
4. Replication is initialised so subsequent syncs are incremental

### What happens on subsequent runs

osmprj detects that the source is already imported and runs `osm2pgsql-replication update` instead — applying only the changes since the last sync. This is much faster than a full re-import.
