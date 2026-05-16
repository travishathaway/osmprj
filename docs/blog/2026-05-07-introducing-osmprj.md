---
title: "Introducing osmprj"
date: 2026-05-07
authors: travishathaway
tags: [design]
---

I'm excited to announce the alpha release of a new tool for managing [OpenStreetMap](https://openstreetmap.org)
(OSM) data with [PostgreSQL](https://postgresql.org): osmprj. This tool aims to reduce the hassle of
working with OSM data in PostgreSQL by providing workflows similar to modern package managers like
[uv](https://docs.astral.sh/uv/), [Cargo](https://crates.io), [npm](https://npmjs.com) or
[pixi](https://pixi.prefix.dev/latest) It's built on top of the tried and true [osm2pgsql](https://osm2pgsql.org)
utility and utilizes the brand  new [osm2pgsql-themepark](https://osm2pgsql.org/themepark)
framework that offers an endless posibility of schema layouts for your OSM data.

In this post, I give you an overview of how this tool works, some background
on how/why I created it and what I wish to accomplish in the future, including some ideas
I have for contributing back to osm2pgsql-themepark.

<!-- truncate -->

## Overview

### Installing

The easiest way to get osmprj is using the custom installer:

```bash
curl -fsSL https://osmprj.dev/install.sh | bash
```

But, it's also possible to install it as a conda package with [pixi](https://pixi.sh):

```bash
pixi global install -c gis-forge -c conda-forge osmprj
```

Or with [conda](https://docs.conda.io):

```bash
conda create -n osmprj -c gis-forge -c conda-forge osmprj
```

### Basic workflow

With osmprj, you can download and set up a PostgreSQL database with just a few commands.

#### 1. Initializing the project

```bash
osmprj init --db postgresql://user@localhost:5432/db
```

#### 2. Adding data sources

Once the project is initialized, you add data sources. These are direct mappings to what's
available on [Geofabrik](https://downloads.geofabrik.com):

```bash
osmprj add monaco
```

You can also specify a custom schema layout with the `--theme` option:

```bash
osmprj add bremen --theme pgosm
```

At this point, your `osmprj.toml` file looks like this:

```toml
[project]
database_url = "postgresql://user@localhost:5432/db"

[sources]
monaco = { schema = "monaco" }
bremen = { schema = "bremen", theme = "pgosm" }
```

#### 3. Syncing to your database

With the `osmprj.toml` defined the way you want, you can download and import
into the data with the following command:

```bash
osmprj sync
```

Because replication is enabled via osm2pgsql, when you want to
update your database later, you simply run `osmprj sync` again.

#### 4. Removing sources

If you want to remove a source from your project, you can use `osmprj remove`:

```bash
osmprj remove monaco
```

This not only removes the source from `osmprj.toml` but also from the database to keep things
in sync and tidy 🧼 🧹.

---

## How this all works

Now that I've given the high-level overview of how the tool works, lets take a peek behind
the curtains to see all the moving pieces.

Like I alluded to in the introduction, this tools is basically a wrapper around osm2pgsql and
utilizes the beta version of osm2pgsql-themepark. I've also created a new osm2pgsql-themepark
theme called [pgosm-themepark](https://github.com/travishathaway/pgosm-themepark), which is
based off of the [pgosm-flex](https://pgosm-flex.com) project.

To make everything easy to install, it is all packaged with conda. The dependencies themselves
are very diverse: osm2pgsql is C/C++ with Python and Lua scripts, plus osmprj itself is written
in Rust. So, conda is actually a perfect fit for this (and no, I'm not just saying that because
I'm one of the conda maintainers!).

Right now, this is being distributed via my own [gis-forge](https://anaconda.org/gis-forge) channel,
but I plan on moving it to the more popular [conda-forge](https://conda-forge.org) channel once
the project stabilizes and leaves alpha. The custom [installer](https://osmprj.dev/install.sh)
was created using [pixi-pack](https://github.com/quantco/pixi-pack).

### Themepark

Right now, osm2pgsql-themepark isn't available in any packaging ecosystem, so part of
getting everything working required me to make updates to this repository to make it more friendly
for packagers. This required the following steps:

1. Create a new `lua/` directory; this holds all the Lua for files in the package.
2. Move `themes/` under this directory in the `themepark/` module.
3. Create a Luarocks `.rockspec` file to hold the metadata for the project enable creating a Lua package.
4. Create a [conda recipe](https://github.com/travishathaway/gis-forge/blob/main/osm2pgsql-themepark/recipe.yaml)
   so this can be published to my gis-forge channel.

All of these changes can be seen in my fork and branch:

- [travishathaway/osm2pgsql-themepark:luarocks](https://github.com/travishathaway/osm2pgsql-themepark/tree/luarocks)

I'll be happy to work with the osm2pgsql-themepark maintainers to see if any of these ideas can be
integrated into the main branch of that project!

### Accessing themes in osmprj

One final piece was missing in order to get themes wired up correctly in osmprj. In osm2pgsql-themepark,
there's a directory called [`config/`](https://github.com/osm2pgsql-dev/osm2pgsql-themepark/tree/master/config)
containing files serving as entry points to their respective themes. Because these
were not included in the Lua package I mentioned above (and they shouldn't be because they're meant to be user
defined configurations), I copied some (but not all) to osmprj in my own [`themes/`](https://github.com/travishathaway/osmprj/tree/main/themes)
directory. There are several different built-in themes for users (e.g. "shortbread" and "pgosm") and
users also have the ability to add their own by appending to the `OSMPRJ_THEME_PATH` environment variable.

To see a full list of all available themes, I've created the following command:

```bash
osmprj themes list
```

I decided to give each osmprj theme its own small `theme.toml` so theme creators can easily add
metadata to them. These themes also support custom SQL scripts that can be run post-import.

Here's an example of what a `theme.toml` looks like:

```toml
name        = "shortbread"
version     = "0.1.0"
description = "Shortbread theme"
type        = "themepark"
entry       = "config.lua"
```

---

## Motivation

There are already so many tools for managing OpenStreetMap data out there. Why does the world
need another one? I believe that `osm2pgsql` itself is a great tool and does its job incredibly
well, but it lacks important features that I have come to love and appreciate by working with
package management tools like [npm](https://npmjs.com), [uv](https://docs.astral.sh/uv/) and
[pixi](https://pixi.prefix.dev/latest/). On top of managing your development environment,
those package managers also handle downloading and caching packages while focusing on delivering
an amazing user experience and being feature rich. I feel like osm2pgsql should be seen as just
a single building block to create an even better user experience.

While looking for such a tool, I came across [PgOSM Flex](https://pgosm-flex.com). This tool
handles downloading data from Geofabrik and also provides a very easy to reason about schema
for working with OSM data in the database. The downside is that this tool needs to be run
inside a Docker container. While using it, I had problems on computers with lower
resources, which is what led me to [forking it](https://github.com/travishathaway/pgosm-flex/tree/experimental-no-docker-setup)
and trying to create a version of it that could be run entirely within conda environments.
But, I ended up getting pretty carried away with this fork, and before I knew it, I had
changed so much, I figured it would be better to just write my own tool (and thus the
[xkcd: 927](https://xkcd.com/927/) cycle begins once more).

While experimenting with PgOSM-Flex, I came up with the idea to add a project configuration
file to the tool (something else I borrowed from popular package managers). With this configuration
file, I envisioned something that could be checked in alongside the code you write to do
your data analysis so that it becomes easy to share and duplicate your work across different
computers and environments.

My final reason for creating this is that I genuinely like writing developer tools. It is
what brings me so much joy being a conda maintainer, and I wanted to begin using this experience
in different problem domains. I really enjoy working with GIS and the types of analysis you
can conduct, especially with OSM data, so combing these two interests with this project felt
like a natural fit.

---

## What's next?

Now that I've reached an initial MVP, I'm going to begin using it myself. If
you've read this far, maybe you would be interested to help me test it and suggest your own
features and improvements? Contributors are welcome too!

I'm hoping I can build something that is at the very least useful for my immediate needs
and perhaps even useful for others as well. Time will tell...
