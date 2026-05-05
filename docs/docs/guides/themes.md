---
id: themes
title: Themes
sidebar_position: 3
---

# Themes

osmprj uses `osm2pgsql` flex Lua styles called *themes* to control how OSM data is structured in your database. You specify a theme when adding a source with `osmprj add --theme <name>`.

## Available themes

| Theme | Description |
|---|---|
| `shortbread` | Shortbread vector tile schema |
| `shortbread-gen` | Shortbread with generalisation |
| `osmcarto` | OSM Carto rendering schema |
| `pgosm` | PgOSM Flex default |
| `pgosm-basic` | PgOSM Flex basic |
| `pgosm-everything` | PgOSM Flex everything |
| `pgosm-minimal` | PgOSM Flex minimal |
| `generic` | Generic basic topics |
| `nwr` | Node/Way/Relation |

Run `osmprj themes list` to see all themes discovered on your system.

## Custom themes

You can add custom theme directories by setting the `OSMPRJ_THEME_PATH` environment variable:

```bash
export OSMPRJ_THEME_PATH="$OSMPRJ_THEME_PATH:/path/to/your/themes"
```

<!-- TODO: expand with theme package layout, theme.toml format, flex vs themepark types -->
