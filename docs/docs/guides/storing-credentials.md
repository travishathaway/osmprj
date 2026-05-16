---
id: storing-credentials
title: Storing Credentials Securely
sidebar_position: 2
---

# Storing Credentials Securely

osmprj needs a PostgreSQL connection URL to function, but storing a password in `osmprj.toml` is risky if the file ends up in version control. To overcome this, the CLI supports several ways to provide the URL:

## `.pgpass` file

A [.pgpass](https://www.postgresql.org/docs/current/libpq-pgpass.html) file is recognized by both osmprj and osm2pgsql. It's the recommend way for storing sensitive database connection details.

Edit your `.pgpass` (`$HOME/.pgpass` for unix-like or `%APPDATA%\postgresql\pgpass.conf` for Windows) file to hold your password:

```
localhost:5432:db:user:pass
```

You can now store the non-sensitive connection details in your `osmprj.toml` file:

```toml
[project]
database_url = "postgresql://user@localhost:5432/db"
```

The credentials you defined in `.pgpass` will automatically be used for you.

## Environment variables

You can also define `OSMPRJ_DATABASE_URL` (e.g. `postgresql://user:password@localhost/db`) and osmprj will use it for all database connections.  This environment variable takes highest priority and overrides everything in `osmprj.toml`. It is never written to disk by osmprj.

```bash
export OSMPRJ_DATABASE_URL="postgres://user:pass@localhost/osm"
```

## `.env` file

An `.env` file can also be used to define the database connection string. A common setup is placing this file next to your `osmprj.toml` file and adding it to your `.gitignore` file to make sure it's not checked in to source control. The CLI loads it automatically at startup, so no shell configuration is required.

```bash
# .env  ← add this to .gitignore
OSMPRJ_DATABASE_URL=postgres://user:pass@localhost/osm
```

```bash
# .gitignore
.env
```

:::tip
All variables in `.env` are loaded into osmprj's environment, so `OSMPRJ_THEME_PATH`, `NO_COLOR`, and any other supported env vars work here too. These values do not override existing values in your environment though.
:::

## Using `osmprj.toml`

The inline URL is still supported as a fallback and is fine when there is no password (common with local PostgreSQL using trust authentication):

```toml
[project]
database_url = "postgres://postgres@localhost/osm"
```

If you must include a password, ensure `osmprj.toml` is in your `.gitignore`.
