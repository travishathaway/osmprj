## ADDED Requirements

### Requirement: Themepark tarball downloaded and cached on first sync
If the themepark cache directory does not exist at `<cache_dir>/osmprj/themepark/`, the tool SHALL download the osm2pgsql-themepark master tarball from GitHub, extract it, and record `cached_at` in `osmprj.lock`. The download SHALL happen before any imports begin.

#### Scenario: First sync, no cache
- **WHEN** themepark cache directory is absent
- **THEN** the tarball is downloaded, extracted, and `osmprj.lock` updated with `cached_at`

#### Scenario: Cache already present
- **WHEN** themepark cache directory exists
- **THEN** no download occurs and the cached copy is used

### Requirement: Direct config file used when no topic customization
When a source's theme maps to a config file in the cached themepark `config/` directory and the source has no `[topics]` block, the config file path SHALL be passed directly as `--style` to osm2pgsql with no tempfile generated.

#### Scenario: No topic customization
- **WHEN** source has `theme = "shortbread_v1"` and no `[topics]` block
- **THEN** `--style` points to the cached `config/shortbread.lua` (or `shortbread_gen.lua`) file

### Requirement: Lua tempfile generated when topics are customized
When a source has a `[topics]` block with `add`, `remove`, or `list` entries, a `NamedTempFile` SHALL be generated containing the `package.path` header, `require('themepark')`, and the resolved topic list. The tempfile SHALL be deleted automatically when the import subprocess exits.

#### Scenario: Topics add
- **WHEN** source has `topics.add = ["core/clean-tags"]`
- **THEN** a tempfile lua is generated that includes the base theme topics plus the additional topic

#### Scenario: Topics list override
- **WHEN** source has `topics.list = ["basic/generic-points"]`
- **THEN** a tempfile lua is generated with exactly that one topic and no others

#### Scenario: Tempfile cleaned up
- **WHEN** osm2pgsql subprocess exits (success or failure)
- **THEN** the tempfile no longer exists on disk

### Requirement: Theme name maps to config filename
The tool SHALL maintain a mapping from theme names used in `osmprj.toml` to config filenames in the themepark `config/` directory. If a theme name has no mapping entry, the tool SHALL fall back to listing `config/*.lua` files to find a match.

#### Scenario: Known theme name
- **WHEN** theme is `shortbread_v1`
- **THEN** the config file `config/shortbread.lua` is used

#### Scenario: Unknown theme name fallback
- **WHEN** theme name has no mapping but a matching `.lua` file exists in `config/`
- **THEN** that file is used

#### Scenario: Theme not found
- **WHEN** theme name has no mapping and no matching file exists
- **THEN** the command exits non-zero with a clear error
