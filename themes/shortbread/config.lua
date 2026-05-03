-- ---------------------------------------------------------------------------
--
-- Shortbread theme
--
-- Configuration for the osm2pgsql Themepark framework
--
-- ---------------------------------------------------------------------------

-- Set these to true in order to create a config or taginfo file
-- If you are creating a tilekiln config you must also create
-- the 'shortbread_config' directory.
local TREX = false
local BBOX = false
local TILEKILN = false
local TAGINFO = false

local themepark = require('themepark')

themepark:set_option('schema', os.getenv('OSMPRJ_SCHEMA') or 'public')
themepark:set_option('srid',   tonumber(os.getenv('OSMPRJ_SRID')) or 3857)

-- For debug mode set this or the environment variable THEMEPARK_DEBUG.
--themepark.debug = true

-- Add JSONB column `tags` with original OSM tags in debug mode
themepark:set_option('tags', 'all_tags')

-- ---------------------------------------------------------------------------
-- Choose which names from which languages to use in the map.
-- See 'themes/core/README.md' for details.

-- themepark:add_topic('core/name-single', { column = 'name' })
-- themepark:add_topic('core/name-list', { keys = {'name', 'name:de', 'name:en'} })

themepark:add_topic('core/name-with-fallback', {
    keys = {
        name = { 'name', 'name:en', 'name:de' },
        name_de = { 'name:de', 'name', 'name:en' },
        name_en = { 'name:en', 'name', 'name:de' },
    }
})

-- --------------------------------------------------------------------------

themepark:add_topic('core/layer')

themepark:add_topic('external/oceans', { name = 'ocean' })

themepark:add_topic('shortbread_v1/aerialways')
themepark:add_topic('shortbread_v1/boundaries')
themepark:add_topic('shortbread_v1/boundary_labels')
themepark:add_topic('shortbread_v1/bridges')
themepark:add_topic('shortbread_v1/buildings')
themepark:add_topic('shortbread_v1/dams')
themepark:add_topic('shortbread_v1/ferries')
themepark:add_topic('shortbread_v1/land')
themepark:add_topic('shortbread_v1/piers')
themepark:add_topic('shortbread_v1/places')
themepark:add_topic('shortbread_v1/pois')
themepark:add_topic('shortbread_v1/public_transport')
themepark:add_topic('shortbread_v1/sites')
themepark:add_topic('shortbread_v1/streets')
themepark:add_topic('shortbread_v1/water')

-- Must be after "pois" layer, because as per Shortbread spec addresses that
-- are already in "pois" should not be in the "addresses" layer.
themepark:add_topic('shortbread_v1/addresses')

-- ---------------------------------------------------------------------------

-- Create config files only in create mode, not when updating the database.
-- This protects the file in case it contains manual edits.
if osm2pgsql.mode == 'create' then
    if TREX then
        themepark:plugin('t-rex'):write_config('t-rex-config.toml', {})
    end

    if BBOX then
        themepark:plugin('bbox'):write_config('bbox-config.toml', {})
    end

    if TILEKILN then
        themepark:plugin('tilekiln'):write_config('shortbread_config', {
            tileset = 'shortbread_v1',
            name = 'OpenStreetMap Shortbread',
            attribution = '<a href="https://www.openstreetmap.org/copyright">© OpenStreetMap</a>'
        })
    end

    if TAGINFO then
        themepark:plugin('taginfo'):write_config('taginfo-shortbread', {
            project = { name = 'shortbread' }
        })
    end
end

-- ---------------------------------------------------------------------------
