## ADDED Requirements

### Requirement: Tuner computes cache size from RAM and PBF size
The tuner SHALL accept system RAM (GB), PBF file size (GB), and the number of concurrent imports and compute an appropriate `--cache` value in MB. The effective RAM budget per import SHALL be `system_ram_gb / concurrent_imports`. The cache formula SHALL be: `cache_mb = min(slim_cache, cache_max) * 1024`, where `slim_cache = 0.75 * (1 + 2.5 * pbf_gb)` and `cache_max = (system_ram_gb / concurrent_imports) * 0.66`.

#### Scenario: Single import uses full RAM budget
- **WHEN** PBF is 0.5 GB, system has 16 GB RAM, and `concurrent_imports = 1`
- **THEN** cache is computed as `min(slim_cache, 16.0 * 0.66) * 1024` (same as before)

#### Scenario: Two concurrent imports halve the RAM budget
- **WHEN** PBF is 0.5 GB, system has 16 GB RAM, and `concurrent_imports = 2`
- **THEN** cache is computed as `min(slim_cache, 8.0 * 0.66) * 1024`

#### Scenario: Cache capped by per-import RAM budget
- **WHEN** the computed slim_cache exceeds the per-import cache_max
- **THEN** cache is set to `floor(cache_max * 1024)` MB where cache_max uses the divided budget
