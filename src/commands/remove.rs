use crate::config::ProjectConfig;
use crate::error::OsmprjError;
use crate::{db, lock::LockFile};
use miette::NamedSource;
use std::fs;
use std::io::{self, Write};
use toml_edit::DocumentMut;

pub async fn run(
    sources: Vec<String>,
    dry_run: bool,
    force: bool,
    config: &ProjectConfig,
) -> Result<(), OsmprjError> {
    // Validate all names before touching anything.
    for name in &sources {
        if !config.sources.contains_key(name.as_str()) {
            return Err(OsmprjError::SourceNotFound { name: name.clone() });
        }
    }

    // Collect schema names for the summary.
    let schema_names: Vec<(String, String)> = sources
        .iter()
        .map(|name| {
            let schema = config.sources[name.as_str()].effective_schema(name);
            (name.clone(), schema)
        })
        .collect();

    if dry_run {
        println!("Dry run — no changes will be made.\n");
        for (name, schema) in &schema_names {
            println!("  Would remove [sources.{name}] from osmprj.toml");
            println!("  Would drop schema '{schema}' from the database");
            println!("  Would remove {name} from osmprj.lock");
        }
        return Ok(());
    }

    if !force {
        println!("The following sources will be permanently removed:\n");
        for (name, schema) in &schema_names {
            println!("  source : {name}");
            println!("  schema : {schema}");
            println!();
        }
        println!("This will drop all database schemas listed above (CASCADE) and cannot be undone.");
        print!("Continue? [y/N] ");
        io::stdout().flush()?;

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Load config file for toml_edit round-trip.
    let raw = fs::read_to_string("osmprj.toml")?;
    let mut doc: DocumentMut = raw.parse().map_err(|e: toml_edit::TomlError| {
        OsmprjError::BadConfig {
            message: e.to_string(),
            src: NamedSource::new("osmprj.toml", raw.clone()),
            span: e.span().map(Into::into),
        }
    })?;

    // Connect to DB once (best-effort).
    let db_client = match config.project.database_url.as_deref() {
        None => None,
        Some(url) => match db::connect(url).await {
            Ok(client) => Some(client),
            Err(e) => {
                eprintln!("warning: could not connect to database — schemas will not be dropped");
                eprintln!("  {e}");
                None
            }
        },
    };

    let mut lock = LockFile::load()?;

    for (name, schema) in &schema_names {
        // 1. Remove from osmprj.toml.
        if let Some(sources_table) = doc.get_mut("sources").and_then(|v| v.as_table_mut()) {
            sources_table.remove(name.as_str());
        }
        println!("Removed [sources.{name}] from osmprj.toml");

        // 2. Drop database schema (best-effort).
        if config.project.database_url.is_none() {
            println!(
                "  hint: no database_url configured — skipping schema drop for '{schema}'"
            );
        } else if let Some(ref client) = db_client {
            match db::drop_schema(client, schema).await {
                Ok(()) => println!("  Dropped schema '{schema}'"),
                Err(e) => eprintln!("  warning: failed to drop schema '{schema}': {e}"),
            }
        }

        // 3. Remove from osmprj.lock (no-op if absent).
        let had_lock_entry = lock.sources.contains_key(name.as_str());
        lock.remove_source(name)?;
        if had_lock_entry {
            println!("  Removed {name} from osmprj.lock");
        }
    }

    // Write osmprj.toml once after all removals.
    fs::write("osmprj.toml", doc.to_string())?;

    Ok(())
}
