-- ---------------------------------------------------------------------------
--
-- Example config for osmcarto theme
--
-- Configuration for the osm2pgsql Themepark framework
--
-- ---------------------------------------------------------------------------

local themepark = require('themepark')

themepark:set_option('schema', os.getenv('OSMPRJ_SCHEMA') or 'public')
themepark:set_option('srid',   tonumber(os.getenv('OSMPRJ_SRID')) or 3857)

-- ---------------------------------------------------------------------------

themepark:add_topic('osmcarto/osmcarto')

-- ---------------------------------------------------------------------------
