use crate::config::{ProjectConfig, SourceConfig};
use crate::error::OsmprjError;
use crate::geofabrik;
use crate::{db};
use miette::NamedSource;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

pub async fn run(
    geofabrik_id: Option<String>,
    path: Option<String>,
    name: Option<String>,
    theme: Option<String>,
    schema: Option<String>,
) -> Result<(), OsmprjError> {
    if !Path::new("osmprj.toml").exists() {
        return Err(OsmprjError::ProjectNotFound);
    }

    let (source_name, pbf_path) = match (geofabrik_id, path, name) {
        (Some(id), None, _) => {
            let features = geofabrik::load_index()?;
            if geofabrik::lookup(&id, &features).is_none() {
                return Err(OsmprjError::UnknownGeofabrikId { id });
            }
            (id, None)
        }
        (None, Some(file_path), Some(label)) => (label, Some(file_path)),
        (None, Some(_), None) => return Err(OsmprjError::MissingName),
        _ => return Err(OsmprjError::InvalidArgs),
    };

    let effective_schema = SourceConfig {
        schema: schema.clone(),
        ..Default::default()
    }
    .effective_schema(&source_name);

    let raw = fs::read_to_string("osmprj.toml")?;
    let mut doc: DocumentMut = raw.parse().map_err(|e: toml_edit::TomlError| {
        OsmprjError::BadConfig {
            message: e.to_string(),
            src: NamedSource::new("osmprj.toml", raw.clone()),
            span: e.span().map(Into::into),
        }
    })?;

    if !doc.contains_key("sources") {
        doc["sources"] = Item::Table(Table::new());
    }

    let sources = doc["sources"]
        .as_table_mut()
        .ok_or_else(|| OsmprjError::BadConfig {
            message: "'sources' must be a table".to_string(),
            src: NamedSource::new("osmprj.toml", raw.clone()),
            span: None,
        })?;

    if sources.contains_key(&source_name) {
        return Err(OsmprjError::DuplicateSource { name: source_name });
    }

    let mut inline = InlineTable::new();
    if let Some(p) = pbf_path {
        inline.insert("path", Value::from(p));
    }
    if let Some(t) = theme {
        inline.insert("theme", Value::from(t));
    }
    inline.insert("schema", Value::from(effective_schema.clone()));

    sources.insert(&source_name, Item::Value(Value::InlineTable(inline)));
    fs::write("osmprj.toml", doc.to_string())?;
    println!("Added [sources.{source_name}] to osmprj.toml");

    // Best-effort: create the schema in the database if a URL is configured.
    let config = ProjectConfig::load()?.unwrap_or_default();
    match config.project.database_url.as_deref() {
        None => {
            println!(
                "  hint: no database_url set — run 'osmprj init --db <url>' or add it to \
                 osmprj.toml to create the schema automatically"
            );
        }
        Some(url) => match db::connect(url).await {
            Err(e) => {
                eprintln!("  warning: could not connect to database — schema '{effective_schema}' was not created");
                eprintln!("  {e}");
                eprintln!(
                    "  hint: check that PostgreSQL is running and that database_url is correct,\n\
                     \t then run 'osmprj status' to verify the connection"
                );
            }
            Ok(client) => match db::create_schema(&client, &effective_schema).await {
                Ok(()) => println!("  created schema '{effective_schema}'"),
                Err(e) => eprintln!("  warning: schema creation failed: {e}"),
            },
        },
    }

    Ok(())
}
