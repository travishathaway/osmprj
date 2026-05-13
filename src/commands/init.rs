use crate::error::OsmprjError;
use std::fs;
use std::path::Path;

pub fn run(db: Option<String>, data_dir: Option<String>) -> Result<(), OsmprjError> {
    if Path::new("osmprj.toml").exists() {
        return Err(OsmprjError::ProjectExists);
    }

    let mut content = String::from("[project]\n");

    if let Some(url) = db {
        content.push_str(&format!("database_url = \"{url}\"\n"));
    } else {
        content.push_str("# database_url = \"postgres://user:pass@localhost/osm\"\n");
    }

    if let Some(dir) = data_dir {
        content.push_str(&format!("data_dir = \"{dir}\"\n"));
    }

    fs::write("osmprj.toml", &content)?;
    println!("Created osmprj.toml");
    Ok(())
}
