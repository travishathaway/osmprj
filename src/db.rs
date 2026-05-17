use crate::error::OsmprjError;
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{ConnectOptions, Row};
use std::str::FromStr;

pub async fn connect(database_url: &str) -> Result<PgConnection, OsmprjError> {
    let opts = PgConnectOptions::from_str(database_url).map_err(|e| {
        OsmprjError::DatabaseConnectFailed {
            message: e.to_string(),
            url: database_url.to_string(),
        }
    })?;

    opts.connect()
        .await
        .map_err(|e| OsmprjError::DatabaseConnectFailed {
            message: e.to_string(),
            url: database_url.to_string(),
        })
}

pub async fn schema_exists(conn: &mut PgConnection, schema: &str) -> Result<bool, OsmprjError> {
    let row = sqlx::query("SELECT 1 FROM information_schema.schemata WHERE schema_name = $1")
        .bind(schema)
        .fetch_optional(conn)
        .await
        .map_err(|e| OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        })?;
    Ok(row.is_some())
}

pub async fn source_is_updatable(
    conn: &mut PgConnection,
    schema: &str,
) -> Result<bool, OsmprjError> {
    let sql =
        format!("SELECT value FROM \"{schema}\".osm2pgsql_properties WHERE property = 'updatable'");
    let row = sqlx::query(&sql).fetch_optional(conn).await.map_err(|e| {
        OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        }
    })?;
    Ok(row
        .map(|r| r.try_get::<String, _>(0).ok().as_deref() == Some("true"))
        .unwrap_or(false))
}

pub async fn create_schema(conn: &mut PgConnection, schema: &str) -> Result<(), OsmprjError> {
    let sql = format!("CREATE SCHEMA IF NOT EXISTS \"{schema}\"");
    sqlx::query(&sql)
        .execute(conn)
        .await
        .map_err(|e| OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        })?;
    Ok(())
}

pub async fn drop_schema(conn: &mut PgConnection, schema: &str) -> Result<(), OsmprjError> {
    let sql = format!("DROP SCHEMA IF EXISTS \"{schema}\" CASCADE");
    sqlx::query(&sql)
        .execute(conn)
        .await
        .map_err(|e| OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        })?;
    Ok(())
}
