## ADDED Requirements

### Requirement: data_dir configures PBF download location
`ProjectSettings` SHALL have an optional `data_dir` field. When set, downloaded `.osm.pbf` files SHALL be saved to `<data_dir>/`. When absent, the default SHALL be `<os_cache_dir>/osmprj/geofabrik/`.

#### Scenario: Custom data_dir
- **WHEN** `osmprj.toml` contains `data_dir = "/mnt/bigdisk/osm"`
- **THEN** PBF files are downloaded to `/mnt/bigdisk/osm/`

#### Scenario: Default data_dir
- **WHEN** `osmprj.toml` has no `data_dir`
- **THEN** PBF files are saved to `~/.cache/osmprj/geofabrik/`

### Requirement: log_dir configures import log location
`ProjectSettings` SHALL have an optional `log_dir` field. When set, osm2pgsql log files SHALL be written to `<log_dir>/`. When absent, the default SHALL be `./logs` (relative to CWD).

#### Scenario: Custom log_dir
- **WHEN** `osmprj.toml` contains `log_dir = "/var/log/osmprj"`
- **THEN** log files are written to `/var/log/osmprj/`

#### Scenario: Default log_dir
- **WHEN** `osmprj.toml` has no `log_dir`
- **THEN** log files are written to `./logs/`

### Requirement: ssd flag influences flat-nodes threshold
`ProjectSettings` SHALL have an optional `ssd` boolean field defaulting to `true`. When `false`, the flat-nodes threshold changes from 8 GB to 30 GB.

#### Scenario: ssd = false raises flat-nodes threshold
- **WHEN** `osmprj.toml` contains `ssd = false` and PBF is 10 GB
- **THEN** `--flat-nodes` is not used (threshold is 30 GB on non-SSD)
