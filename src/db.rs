use crate::error::OsmprjError;
use tokio_postgres::{Client, NoTls};

pub async fn connect(database_url: &str) -> Result<Client, OsmprjError> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls)
        .await
        .map_err(|e| OsmprjError::DatabaseConnectFailed {
            message: e.to_string(),
            url: database_url.to_string(),
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("database connection error: {e}");
        }
    });

    Ok(client)
}

pub async fn schema_exists(client: &Client, schema: &str) -> Result<bool, OsmprjError> {
    let row = client
        .query_opt(
            "SELECT 1 FROM information_schema.schemata WHERE schema_name = $1",
            &[&schema],
        )
        .await
        .map_err(|e| OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        })?;
    Ok(row.is_some())
}

pub async fn source_is_updatable(client: &Client, schema: &str) -> Result<bool, OsmprjError> {
    let sql =
        format!("SELECT value FROM \"{schema}\".osm2pgsql_properties WHERE property = 'updatable'");
    let row = client.query_opt(sql.as_str(), &[]).await.map_err(|e| {
        OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        }
    })?;
    Ok(row.map(|r| r.get::<_, &str>(0) == "true").unwrap_or(false))
}

pub async fn create_schema(client: &Client, schema: &str) -> Result<(), OsmprjError> {
    // Schema names are produced by effective_schema() which normalises to
    // [a-z0-9_], or are supplied by --schema. Quoting with " " is sufficient
    // to prevent injection while allowing any identifier the user might provide.
    let sql = format!("CREATE SCHEMA IF NOT EXISTS \"{schema}\"");
    client
        .execute(sql.as_str(), &[])
        .await
        .map_err(|e| OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        })?;
    Ok(())
}

pub async fn drop_schema(client: &Client, schema: &str) -> Result<(), OsmprjError> {
    let sql = format!("DROP SCHEMA IF EXISTS \"{schema}\" CASCADE");
    client
        .execute(sql.as_str(), &[])
        .await
        .map_err(|e| OsmprjError::DatabaseQueryFailed {
            message: e.to_string(),
        })?;
    Ok(())
}
