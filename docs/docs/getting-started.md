---
id: getting-started
title: Getting Started
sidebar_position: 2
---

# Getting Started

## Installation

The easiest way to install osmprj is with our custom installer:

```bash
curl -fsSL https://osmprj.dev/install.sh | bash
```

:::tip
Want to verify the script first? Check out the **[latest release on GitHub](https://github.com/travishathaway/osmprj/releases/latest)**.
:::

### Install as a conda package

It's also possible to install osmprj as a conda package from the [gis-forge](https://anaconda.org/gis-forge) channel.

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

osmprj requires a running [PostgreSQL](https://postgresql.org) instance with the [PostGIS](https://postgis.net/) extension installed.

## Quick Start

The typical workflow is three commands:

```bash
# 1. Create a project file in the current directory
osmprj init --db postgresql://user@localhost:5432/db

# 1a. Optional: If using a password, save to an .env file
echo "OSMPRJ_DATABASE_URL=postgresql://user:pass@localhost:5432/db" > .env

# 2. Add a Geofabrik region
osmprj add germany --theme shortbread

# 3. Download and import the data
osmprj sync
```

:::warning
**Avoid storing database passwords in `osmprj.toml`.** If this file is committed to version control, it could risk exposing your credentials. See the [Storing Credentials Securely](guides/storing-credentials) guide for more information on best practices.
:::

### What happens on first run

When you run `osmprj sync` for the first time:

1. The PBF file is downloaded from Geofabrik with a progress bar (MD5-verified)
2. `osm2pgsql` performs a full import into your database
3. Replication is initialized so subsequent syncs are incremental

### What happens on subsequent runs

osmprj detects that the source is already imported and runs `osm2pgsql-replication update` instead — applying only the changes since the last sync. This is much faster than a full re-import.
