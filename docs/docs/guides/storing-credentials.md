---
id: storing-credentials
title: Storing Credentials Securely
sidebar_position: 2
---

# Storing Credentials Securely

osmprj needs a PostgreSQL connection URL to function, but storing a password in `osmprj.toml` is risky if the file ends up in version control. The CLI supports four ways to provide the URL, checked in this order:

## Environment variables

You can define `OSMPRJ_DATABASE_URL` (e.g. `postgresql://user:password@localhost/db`) and osmprj will use it for all database connections.  This environment variable takes highest priority and overrides everything in `osmprj.toml`. It is never written to disk by osmprj.

```bash
export OSMPRJ_DATABASE_URL="postgres://user:pass@localhost/osm"
```

## `.env` files

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

## Custom retrieval command

You can also define a custom command in your `osmprj.toml` file under `project.database_url_command` to retrieve passwords at runtime (recommended for desktops and shared configs).

Add a shell command to your `osmprj.toml`, and the CLI runs the command and reads the full database URL from its stdout. The command runs via `sh -c` on Unix or `cmd /C` on Windows.

```toml
[project]
database_url_command = "pass show osmprj/db-url"
```

This keeps the password out of the config file entirely — only a reference to your secret manager is stored. Some real-world examples:

**[pass](https://www.passwordstore.org/) — the standard Unix password manager (GPG-backed):**
```toml
database_url_command = "pass show osmprj/db-url"
```

**[1Password CLI](https://developer.1password.com/docs/cli/):**
```toml
database_url_command = "op read op://Personal/osmprj-db/url"
```

**GPG-encrypted file:**
```toml
database_url_command = "gpg --quiet --decrypt ~/.osmprj-db-url.gpg"
```

**AWS Secrets Manager:**
```toml
database_url_command = "aws secretsmanager get-secret-value --secret-id osmprj/db --query SecretString --output text"
```

**HashiCorp Vault:**
```toml
database_url_command = "vault kv get -field=url secret/osmprj/db"
```

:::tip
In headless/CI environments, commands must be non-interactive (no passphrase prompts). Use the `OSMPRJ_DATABASE_URL` env var or `.env` file instead if your secret manager requires user interaction.
:::

## Using `osmprj.toml`

The inline URL is still supported as a fallback and is fine when there is no password (common with local PostgreSQL using trust authentication):

```toml
[project]
database_url = "postgres://postgres@localhost/osm"
```

If you must include a password, ensure `osmprj.toml` is in your `.gitignore`.
