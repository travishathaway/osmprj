---
id: themes-list
title: themes list
sidebar_position: 6
---

# `osmprj themes list`

```
osmprj themes list
```

Lists all available themes discovered on the current system.

## Built-in themes

| Theme | Description |
|---|---|
| `generic` | Generic basic topics theme |
| `nwr` | Node/Way/Relation (NWR) topic theme |
| `osmcarto` | OSM Carto theme |
| `pgosm` | PgOSM Flex theme: default variation |
| `pgosm-basic` | PgOSM Flex theme: basic variation |
| `pgosm-everything` | PgOSM Flex theme: everything variation |
| `pgosm-minimal` | PgOSM Flex theme: minimal variation |
| `shortbread` | Shortbread theme |
| `shortbread-gen` | Shortbread theme with generalization |

## Custom themes

To make additional theme directories discoverable, append to the `OSMPRJ_THEME_PATH` environment variable:

```bash
export OSMPRJ_THEME_PATH="$OSMPRJ_THEME_PATH:/your/themes"
```

See the [Themes](/docs/guides/themes) guide for more details.
