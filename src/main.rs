mod commands;
mod config;
mod db;
mod error;
mod geofabrik;
mod lock;
mod output;
mod theme_registry;
mod tuner;

use clap::{Args, ColorChoice, CommandFactory, FromArgMatches, Parser, Subcommand};
use config::ProjectConfig;

#[derive(Args, Clone)]
struct ColorArgs {
    /// Force colored output (overrides NO_COLOR env var)
    #[arg(long, global = true, conflicts_with = "no_color")]
    color: bool,
    /// Disable colored output
    #[arg(long, global = true, conflicts_with = "color")]
    no_color: bool,
}

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

    #[command(flatten)]
    color: ColorArgs,

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

/// Scan raw args and env to determine clap's ColorChoice before full parsing.
/// This runs before `Cli::parse()` so that `--help` output respects the flags.
fn early_color_choice() -> ColorChoice {
    let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--no-color") {
        ColorChoice::Never
    } else if args.iter().any(|a| a == "--color") {
        ColorChoice::Always
    } else if no_color_env {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    }
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

    let matches = Cli::command()
        .color(early_color_choice())
        .styles(output::help_styles())
        .get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    // Apply color settings: CLI flags win, then NO_COLOR env var, then console defaults.
    if cli.color.no_color {
        console::set_colors_enabled(false);
        console::set_colors_enabled_stderr(false);
    } else if cli.color.color {
        console::set_colors_enabled(true);
        console::set_colors_enabled_stderr(true);
    } else if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        console::set_colors_enabled(false);
        console::set_colors_enabled_stderr(false);
    }

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
