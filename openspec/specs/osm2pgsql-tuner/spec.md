## ADDED Requirements

### Requirement: Tuner computes cache size from RAM and PBF size
The tuner SHALL accept system RAM (GB) and PBF file size (GB) and compute an appropriate `--cache` value in MB using the formula: `cache_mb = min(slim_cache, cache_max) * 1024`, where `slim_cache = 0.75 * (1 + 2.5 * pbf_gb)` and `cache_max = system_ram_gb * 0.66`.

#### Scenario: Small PBF with ample RAM
- **WHEN** PBF is 0.5 GB and system has 16 GB RAM
- **THEN** cache is set to `floor(0.75 * 1 + 2.5 * 0.5) * 1024)` = 1689 MB

#### Scenario: Cache capped by available RAM
- **WHEN** the computed slim_cache exceeds cache_max
- **THEN** cache is set to `floor(cache_max * 1024)` MB

### Requirement: Flat-nodes used for large files
The tuner SHALL set `--flat-nodes=<data_dir>/<source_name>.nodes` and `--cache=0` when PBF size is ≥ 8.0 GB and `ssd=true`, or when PBF size is ≥ 30.0 GB regardless of storage type.

#### Scenario: Large file on SSD
- **WHEN** PBF is 10 GB and `ssd = true`
- **THEN** `--flat-nodes` is included and `--cache=0`

#### Scenario: Very large file on spinning disk
- **WHEN** PBF is 35 GB and `ssd = false`
- **THEN** `--flat-nodes` is included and `--cache=0`

#### Scenario: Small file, flat-nodes not used
- **WHEN** PBF is 2 GB and `ssd = true`
- **THEN** `--flat-nodes` is not included

### Requirement: Command always uses slim and create mode
Every generated osm2pgsql command SHALL include `--slim` and `--create`. `--drop` SHALL never be included. This ensures middle tables are retained for subsequent replication updates.

#### Scenario: Generated command structure
- **WHEN** tuner builds a command for any source
- **THEN** the command contains `--slim` and `--create` and does not contain `--drop`

### Requirement: Database and schema flags always present
The generated command SHALL always include `--database=<database_url>` and `--schema=<effective_schema>`.

#### Scenario: Database and schema in command
- **WHEN** tuner builds a command for source `albania` with schema `albania`
- **THEN** the command includes `--database=<url>` and `--schema=albania`
