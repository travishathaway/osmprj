# OSM Project Manager (osmprj)

This is a command line tool for working with OSM data, specifically preparing it for import into PostgreSQL
and then creating some reports from it.

It was originally written for a PyConDE talk I gave in 2022:

Full video of the presentation:

*Coming soon!*

Slides used in the presentation

[Slides link](https://docs.google.com/presentation/d/1nFQr_NUr-QmG0n-wusnctjnAl8YUKmMuz3PhU_FoTYo/edit?usp=sharing)

Full blog article going in depth on the rationale behind this project:

[Processing OpenStreetMap data with PostgreSQL and Python](https://travishathaway.com/posts/2022-04-02-processing-osm-data-with-postgresql-and-python/)

With that being said, it was originally thought of as an example project from which others could
begin their own. This repo will continue to grow and accommodate my use cases, but I hope it will continue to
serve its original purposes as an example project.

## Getting started

This project is managed via poetry. To begin using run the following commands:

```bash
$ poetry install 
# ... Installs dependencies
$ poetry shell
# ... Activates virtual environment
```

## Other requirements

- osmium (^1.1.1)
- osm2pgsql (^1.6.0) (optional, only used for importing data into PostgreSQL)

## osmprj CLI

Below are the commands for this CLI:

### `osmprj prepare`

```
Usage: osmprj prepare [OPTIONS] OSM_DATA_DESC OUTPUT

  Retrieves an OSM data file and runs an optional extract-and-merge step to
  provide us with our project data

Options:
  -c, --config FILENAME  Config file holding bound box extracts
  -d, --dry-run          Do not run any commands, just print what would be
                         performed to stdout
  -s, --silent           Display no stdout output
  --help                 Show this message and exit.
```

### `osmprj report`

```
Usage: osmprj report [OPTIONS] COMMAND [ARGS]...

  These sub-commands are responsible for generating reports

Options:
  --help  Show this message and exit.

Commands:
  amenity_city
  parking_space
```