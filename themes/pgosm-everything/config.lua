-- ---------------------------------------------------------------------------
--
-- pgosm-themepark config: everything
--
-- Layerset: all 20 topics
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

themepark:add_topic('pgosm/amenity')
themepark:add_topic('pgosm/building')
themepark:add_topic('pgosm/building_combined_point')
themepark:add_topic('pgosm/indoor')
themepark:add_topic('pgosm/infrastructure')
themepark:add_topic('pgosm/landuse')
themepark:add_topic('pgosm/leisure')
themepark:add_topic('pgosm/natural')
themepark:add_topic('pgosm/place')
themepark:add_topic('pgosm/poi')
themepark:add_topic('pgosm/poi_combined_point')
themepark:add_topic('pgosm/public_transport')
themepark:add_topic('pgosm/road')
themepark:add_topic('pgosm/road_major')
themepark:add_topic('pgosm/shop')
themepark:add_topic('pgosm/shop_combined_point')
themepark:add_topic('pgosm/tags')
themepark:add_topic('pgosm/traffic')
themepark:add_topic('pgosm/unitable')
themepark:add_topic('pgosm/water')
