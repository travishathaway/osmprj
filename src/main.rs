mod commands;
mod config;
mod db;
mod error;
mod geofabrik;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "osmprj", about = "OpenStreetMap and PostgreSQL project management tool", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new osmprj.toml project file in the current directory
    Init {
        /// Database connection URL
        #[arg(long)]
        db: Option<String>,
    },
    /// Add a new OSM data source to osmprj.toml
    Add {
        /// Geofabrik region ID (e.g. germany, europe/france)
        geofabrik_id: Option<String>,
        /// Path to a local .osm.pbf file
        #[arg(long)]
        path: Option<String>,
        /// Source name/label (required with --path)
        #[arg(long)]
        name: Option<String>,
        /// Themepark theme (e.g. shortbread_v1, basic)
        #[arg(long)]
        theme: Option<String>,
        /// PostgreSQL schema name (defaults to normalized source name)
        #[arg(long)]
        schema: Option<String>,
    },
    /// Sync OSM data sources listed in osmprj.toml to the configured database
    Sync,
    /// Remove a data source from osmprj.toml
    Remove,
    /// Remove all OSM data from the configured database
    Destroy,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().context_lines(3).tab_width(4).build())
    }))
    .ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { db } => commands::init::run(db),
        Commands::Add { geofabrik_id, path, name, theme, schema } => {
            commands::add::run(geofabrik_id, path, name, theme, schema)
        }
        Commands::Sync => {
            println!("sync: not yet implemented");
            Ok(())
        }
        Commands::Remove => {
            println!("remove: not yet implemented");
            Ok(())
        }
        Commands::Destroy => {
            println!("destroy: not yet implemented");
            Ok(())
        }
    }
    .map_err(miette::Report::new)?;

    Ok(())
}
