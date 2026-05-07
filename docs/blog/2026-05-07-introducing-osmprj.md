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
database_url = "postgresql://postgres@localhost:65432/postgres"

[sources]
monaco = { schema = "monaco" }
bremen = { schema = "monaco", theme = "pgosm" }
```

### Syncing to your database

With the `osmprj.toml` defined the way we want, now can download the data and import
into our database:

```commandline
osmprj sync
```

Because replication is enabled via the `osm2pgsql-replication` command, when we want to
update our database later, we simply just run `osmprj sync` again.






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

## Themepark

Another goal I had with this project was to try out the  beta version of osm2pgsql-themepark.
After working with PgOSM-Flex, I could see a benefit for building out a theme system based on
resuable components.
