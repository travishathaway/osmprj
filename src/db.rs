use crate::config::ProjectConfig;
use tokio_postgres::{Client, NoTls};

pub async fn connect(config: &ProjectConfig) -> Result<Client, Box<dyn std::error::Error>> {
    let url = config.project.database_url.as_deref().ok_or(
        "No database URL configured. \
         Set OSMPRJ_DATABASE_URL or add database_url to osmprj.toml.",
    )?;

    let (client, connection) = tokio_postgres::connect(url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("database connection error: {e}");
        }
    });

    Ok(client)
}
