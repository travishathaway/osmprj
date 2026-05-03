mod commands;
mod config;
mod db;
mod error;
mod geofabrik;
mod lock;
mod theme_registry;
mod tuner;

use clap::{Parser, Subcommand};
use config::ProjectConfig;

#[derive(Parser)]
#[command(
    name = "osmprj",
    about = "OpenStreetMap and PostgreSQL project management tool",
    version
)]
struct Cli {
    /// Enable verbose output (stream osm2pgsql logs to terminal)
    #[arg(short = 'v', long, global = true)]
    verbose: bool,

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
        /// Geofabrik region IDs (e.g. germany, europe/france); accepts multiple
        geofabrik_ids: Vec<String>,
        /// Path to a local .osm.pbf file
        #[arg(long)]
        path: Option<String>,
        /// Source name/label (required with --path)
        #[arg(long)]
        name: Option<String>,
        /// Themepark theme (e.g. shortbread_v1, basic) or plugin theme name
        #[arg(long)]
        theme: Option<String>,
        /// PostgreSQL schema name (defaults to normalized source name; cannot be used with multiple IDs)
        #[arg(long)]
        schema: Option<String>,
        /// Spatial reference ID (default: 3857)
        #[arg(long)]
        srid: Option<u32>,
    },
    /// Show project and database status
    Status,
    /// Sync OSM data sources listed in osmprj.toml to the configured database
    Sync {
        /// Specific sources to sync (defaults to all)
        sources: Vec<String>,
    },
    /// Remove a data source from osmprj.toml
    Remove {
        /// Source names to remove
        sources: Vec<String>,
        /// Preview what would be removed without making any changes
        #[arg(long)]
        dry_run: bool,
        /// Skip the confirmation prompt
        #[arg(short = 'f', long)]
        force: bool,
    },
    /// Manage and inspect installed themes
    Themes {
        #[command(subcommand)]
        subcommand: ThemesCommands,
    },
}

#[derive(Subcommand)]
enum ThemesCommands {
    /// List all available themes (plugin and built-in)
    List,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .context_lines(3)
                .tab_width(4)
                .build(),
        )
    }))
    .ok();

    let cli = Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        Commands::Init { db } => commands::init::run(db),
        Commands::Add {
            geofabrik_ids,
            path,
            name,
            theme,
            schema,
            srid,
        } => commands::add::run(geofabrik_ids, path, name, theme, schema, srid).await,
        Commands::Status => {
            let config = ProjectConfig::load()?.ok_or(error::OsmprjError::ProjectNotFound)?;
            commands::status::run(&config).await
        }
        Commands::Sync { sources } => {
            let config = ProjectConfig::load()?.ok_or(error::OsmprjError::ProjectNotFound)?;
            commands::sync::run(sources, verbose, &config).await
        }
        Commands::Remove {
            sources,
            dry_run,
            force,
        } => {
            let config = ProjectConfig::load()?.ok_or(error::OsmprjError::ProjectNotFound)?;
            commands::remove::run(sources, dry_run, force, &config).await
        }
        Commands::Themes {
            subcommand: ThemesCommands::List,
        } => commands::themes::run_list(),
    }
    .map_err(miette::Report::new)?;

    Ok(())
}
