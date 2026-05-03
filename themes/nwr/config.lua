-- ---------------------------------------------------------------------------
--
-- Example config for basic/nwr topic
--
-- Configuration for the osm2pgsql Themepark framework
--
-- ---------------------------------------------------------------------------

local themepark = require('themepark')

themepark:set_option('schema', os.getenv('OSMPRJ_SCHEMA') or 'public')
themepark:set_option('srid',   tonumber(os.getenv('OSMPRJ_SRID')) or 3857)

-- For debug mode set this or the environment variable THEMEPARK_DEBUG.
--themepark.debug = true

-- ---------------------------------------------------------------------------

themepark:add_topic('core/clean-tags')

themepark:add_topic('basic/nwr')

-- ---------------------------------------------------------------------------
