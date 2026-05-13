-- ---------------------------------------------------------------------------
--
-- bad-postprocess: minimal test theme
--
-- Uses a single shortbread_v1 topic so the osm2pgsql import succeeds.
-- The sql/ directory contains intentionally broken SQL to exercise
-- post-processing error handling.
--
-- Runtime env vars (all optional):
--   OSMPRJ_SCHEMA      - PostgreSQL schema name (default: public)
--   OSMPRJ_SRID        - Spatial reference ID (default: 3857)
--
-- ---------------------------------------------------------------------------

local themepark = require('themepark')

themepark:set_option('schema', os.getenv('OSMPRJ_SCHEMA') or 'public')
themepark:set_option('srid',   tonumber(os.getenv('OSMPRJ_SRID')) or 3857)

themepark:add_topic('shortbread_v1/addresses')
