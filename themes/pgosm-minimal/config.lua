-- ---------------------------------------------------------------------------
--
-- pgosm-themepark config: minimal
--
-- Layerset: place, poi_combined_point, road_major
--
-- Runtime env vars (all optional):
--   OSMPRJ_SCHEMA      - PostgreSQL schema name (default: osm)
--   OSMPRJ_SRID        - Spatial reference ID (default: 3857)
--
-- ---------------------------------------------------------------------------

local themepark = require('themepark')

themepark:set_option('schema', os.getenv('OSMPRJ_SCHEMA') or 'osm')
themepark:set_option('srid',   tonumber(os.getenv('OSMPRJ_SRID')) or 3857)

-- Name handling: load core/name-with-fallback BEFORE any pgosm topics
themepark:add_topic('core/name-with-fallback', {
    keys = { name = { 'name', 'short_name', 'alt_name', 'loc_name', 'old_name' } }
})

themepark:add_topic('pgosm/place')
themepark:add_topic('pgosm/poi_combined_point')
themepark:add_topic('pgosm/road_major')
