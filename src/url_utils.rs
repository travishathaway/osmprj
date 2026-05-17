use crate::error::OsmprjError;
use percent_encoding::percent_decode_str;
use url::Url;

/// Parsed PostgreSQL connection parameters extracted from a database URL.
#[derive(Debug, Clone)]
pub struct PgConnParams {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: Option<String>,
}

impl PgConnParams {
    /// Return a credential-free connection URL suitable for logging and for
    /// passing as a `--database` / `-d` argument to child processes.
    /// Format: `postgresql://host:port/database`
    pub fn credential_free_url(&self) -> String {
        format!("postgresql://{}:{}/{}", self.host, self.port, self.database)
    }
}

/// Parse a PostgreSQL connection URL into its component parts.
///
/// Supports `postgresql://` and `postgres://` schemes. Percent-encoded
/// characters in the password are decoded automatically by the `url` crate.
///
/// Returns `Err` if the URL cannot be parsed or is missing required fields.
pub fn parse_db_url(url: &str) -> Result<PgConnParams, OsmprjError> {
    let parsed = Url::parse(url).map_err(|e| OsmprjError::InvalidDatabaseUrl {
        message: e.to_string(),
    })?;

    let host = parsed.host_str().unwrap_or("localhost").to_string();

    let port = parsed.port().unwrap_or(5432);

    // Database name is the path component without the leading slash.
    let database = parsed.path().trim_start_matches('/').to_string();
    let database = if database.is_empty() {
        // Fall back to the username as Postgres does by default.
        parsed.username().to_string()
    } else {
        database
    };

    let user = parsed.username().to_string();

    let password = parsed.password().map(|p| {
        // url::Url::password() returns the percent-encoded string; decode it
        // so that passwords containing special characters (e.g. `@`, `:`) round-trip
        // correctly when passed to child processes via PGPASSWORD.
        percent_decode_str(p).decode_utf8_lossy().into_owned()
    });

    Ok(PgConnParams {
        host,
        port,
        database,
        user,
        password,
    })
}

/// Return a copy of `url` with the password segment replaced by `****`.
/// If the URL cannot be parsed or has no password, the original string is
/// returned unchanged.
pub fn mask_db_url(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return url.to_string();
    };

    if parsed.password().is_none() {
        return url.to_string();
    }

    // set_password only fails if the URL has no host (e.g. data URLs), which
    // can't be a Postgres connection URL, so we can safely ignore the error.
    let _ = parsed.set_password(Some("****"));
    parsed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_db_url ──────────────────────────────────────────────────────────

    #[test]
    fn parse_url_with_password() {
        let params = parse_db_url("postgresql://myuser:mysecret@localhost:5432/mydb").unwrap();
        assert_eq!(params.host, "localhost");
        assert_eq!(params.port, 5432);
        assert_eq!(params.database, "mydb");
        assert_eq!(params.user, "myuser");
        assert_eq!(params.password, Some("mysecret".to_string()));
    }

    #[test]
    fn parse_url_without_password() {
        let params = parse_db_url("postgresql://myuser@localhost:5432/mydb").unwrap();
        assert_eq!(params.user, "myuser");
        assert_eq!(params.password, None);
    }

    #[test]
    fn parse_url_default_port() {
        let params = parse_db_url("postgresql://myuser@localhost/mydb").unwrap();
        assert_eq!(params.port, 5432);
        assert_eq!(params.host, "localhost");
        assert_eq!(params.database, "mydb");
    }

    #[test]
    fn parse_url_percent_encoded_password() {
        // Password is "p@ss:word" — both @ and : must be percent-encoded.
        let params = parse_db_url("postgresql://user:p%40ss%3Aword@localhost:5432/mydb").unwrap();
        assert_eq!(params.password, Some("p@ss:word".to_string()));
    }

    #[test]
    fn parse_url_postgres_scheme() {
        let params = parse_db_url("postgres://user:pass@host:5433/db").unwrap();
        assert_eq!(params.host, "host");
        assert_eq!(params.port, 5433);
    }

    #[test]
    fn credential_free_url_strips_credentials() {
        let params = parse_db_url("postgresql://user:secret@db.example.com:5432/mydb").unwrap();
        assert_eq!(
            params.credential_free_url(),
            "postgresql://db.example.com:5432/mydb"
        );
    }

    // ── mask_db_url ───────────────────────────────────────────────────────────

    #[test]
    fn mask_url_with_password() {
        let masked = mask_db_url("postgresql://user:secret@localhost:5432/mydb");
        assert!(masked.contains("user:****"), "got: {masked}");
        assert!(!masked.contains("secret"), "got: {masked}");
    }

    #[test]
    fn mask_url_without_password() {
        let url = "postgresql://user@localhost:5432/mydb";
        let masked = mask_db_url(url);
        assert_eq!(masked, url);
    }

    #[test]
    fn mask_url_invalid_returns_original() {
        let url = "not-a-url";
        assert_eq!(mask_db_url(url), url);
    }
}
