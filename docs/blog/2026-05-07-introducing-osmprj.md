---
title: "Introducing osmprj"
date: 2026-05-07
authors: travishathaway
tags: [design]
---

I'm excited to announce the alpha release of a new tool for managing OpenStreetMap (OSM) data
with PostgreSQL: osmprj. Osmprj aims to reduce the hassle of manually donwloading
and managing OSM data dumps by providing workflows similar to modern package managers
like [uv](https://docs.astral.sh/uv/), [Cargo](https://crates.io), [npm](https://npmjs.com) or
[pixi](https://pixi.prefix.dev/latest) It's built on top of the tried and true [osm2pgsql](https://osm2pgsql.org)
utility and utilizes the brand  new [osm2pgsql-themepark](https://osm2pgsql.org/themepark)
framework that offers an endless posibility schema layouts for your OSM data.

In this post, I'll give you all an overview of how this tool works, some background
on how/why I created and what I wish to accomplish in the future, including some ideas
I have for contributing back to and improving osm2pgsql-themepark.

<!-- truncate -->

## Overview

### Installing

This easiest way to get osmprj is by installing it as a conda package with [pixi](https://pixi.sh):

```commandline
pixi global install -c gis-forge -c conda-forge osmprj
```

Or with [conda](https://docs.conda.io):

```commandline
conda create -n osmprj -c gis-forge -c conda-forge osmprj
```

### Initializing your project

With osmprj, you can download and set up a PostgreSQL database with just a few commands:

```commandline
osmprj init --db postgresql://user@localhost:5432/db
```

### Adding data sources

Once the project is initialized, you add data sources. These are a direct mapping to what's
available on [Geofabrik](https://downloads.geofabrik.com):

```commandline
osmprj add monaco
```

You can also specify a custom schema layout with the `--theme` option:

```commandline
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

### Syncing to your database

With the `osmprj.toml` defined the way we want, now can download the data and import
into our database:

```commandline
osmprj sync
```

Because replication is enabled via the `osm2pgsql-replication` command, when we want to
update our database later, we simply just run `osmprj sync` again.

### Removing sources

If you to remove a source from your project, you can use `osmprj remove`:

```commandline
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
but I plan on moving it to the more popular [conda-forge](https://conda-forge.org) channel soon.

### Themepark

Right now, the osm2pgsql-themepark isn't actually available in any packaging ecosystem, so part of
getting everything working required me to make updates to this repository to make it more friendly
for packagers. This required the following steps:

1. Create a new `lua/` directory; this holds all the Lua for files in the package.
2. Move `themes/` under this directory in the `themepark/` module.
3. Create a Luarocks `.rockspec` file to hold the metadata for the project enable creating a Lua package.
4. Create a [conda recipe](https://github.com/travishathaway/gis-forge/blob/main/osm2pgsql-themepark/recipe.yaml)
   so this can be published to my gis-forge channel.

All of these changes can be seen in my fork and branch:

- [travishathaway/osm2pgsql-themepark:luarocks](https://github.com/travishathaway/osm2pgsql-themepark/tree/luarocks)

### Themes in osmprj

One final piece was missing in order to get themes wired up correctly in osmprj. In osm2pgsql-themepark,
there's a directory called `config/` that is meant to be the entry point for a using a theme. Because these
were not included in the Lua package I mentioned above (and they shouldn't be because they meant to be user
defined configurations), I add them to osmprj in my own `themes/` directory. There are several differnt
built-in themes for users (e.g. "shortbread" and "pgosm") and users also have the ability to add their
own by appending to the `OSMPRJ_THEME_PATH` environment variable.

I decided to give each osmprj theme its own small `theme.toml` so user can easily add metadata to them.
These themes also technically support custom SQL scripts that can be run post-import.

---

## Motivation

There are already so many tools for managing OpenStreetMap data out there. Why does the world
need another one? I believe that `osm2pgsql` itself is a great tool and does its job incredibly
well, but it lacks important features that I have come to love and appreciate by working with
package management tools like [npm](https://npmjs.com), [uv](https://docs.astral.sh/uv/) and
[pixi](https://pixi.prefix.dev/latest/). On top of managing your development environment,
those package managers also handle downloading packages from a server hosting packages. This
is the key part that `osm2pgsql` is missing and something I believe should be combined
into a single tool.

While looking for such a tool, I came across [PgOSM Flex](https://pgosm-flex.com). This tool
handles downloading data from Geofabrik and also provides a very easy to reason about schema
for working with OSM data in the database. The downside is that this tool needs to be run
inside a Docker container. While using, I especially had problems on computers with lower
resources, which is what led me to [forking it](https://github.com/travishathaway/pgosm-flex/tree/experimental-no-docker-setup)
and trying to create a version of it that could be run entirely within conda environments.
But, I ended up getting pretty carried away with this fork, and before I knew it, I had
changed so much, I figured it would be better to just write my own tool.

While experimenting with PgOSM-Flex, I came up with the idea to add a project configuration
file to the tool (something else I borrowed from popular package managers). With this configuration
file, I envisioned something that could be checked in alongside the code you write to do
your data analysis so that it becomes easy to share and duplicate your work across different
computers and environments.

---

## What's next?

*TBD*
