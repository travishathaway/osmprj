use crate::config::SourceConfig;
use crate::error::OsmprjError;
use crate::geofabrik;
use miette::NamedSource;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

pub fn run(
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

    let source_config = SourceConfig {
        schema,
        ..Default::default()
    };
    let effective_schema = source_config.effective_schema(&source_name);

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
    inline.insert("schema", Value::from(effective_schema));

    sources.insert(&source_name, Item::Value(Value::InlineTable(inline)));

    fs::write("osmprj.toml", doc.to_string())?;
    println!("Added [sources.{source_name}] to osmprj.toml");
    Ok(())
}
