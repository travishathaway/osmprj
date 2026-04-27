use crate::error::OsmprjError;
use miette::NamedSource;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct TopicsConfig {
    pub list: Option<Vec<String>>,
    pub add: Option<Vec<String>>,
    pub remove: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SourceConfig {
    pub path: Option<String>,
    pub theme: Option<String>,
    pub schema: Option<String>,
    pub topics: Option<TopicsConfig>,
}

impl SourceConfig {
    pub fn effective_schema(&self, name: &str) -> String {
        self.schema
            .clone()
            .unwrap_or_else(|| name.replace(['/', '-'], "_"))
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ProjectSettings {
    pub database_url: Option<String>,
    pub data_dir: Option<String>,
    pub log_dir: Option<String>,
    pub ssd: Option<bool>,
    pub max_diff_size_mb: Option<u32>,
}

impl ProjectSettings {
    pub fn effective_data_dir(&self) -> PathBuf {
        if let Some(ref d) = self.data_dir {
            return PathBuf::from(d);
        }
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("osmprj")
            .join("geofabrik")
    }

    pub fn effective_log_dir(&self) -> PathBuf {
        if let Some(ref d) = self.log_dir {
            PathBuf::from(d)
        } else {
            PathBuf::from("logs")
        }
    }

    pub fn effective_ssd(&self) -> bool {
        self.ssd.unwrap_or(true)
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: ProjectSettings,
    #[serde(default)]
    pub sources: HashMap<String, SourceConfig>,
}

impl ProjectConfig {
    pub fn load() -> Result<Option<Self>, OsmprjError> {
        let content = match fs::read_to_string("osmprj.toml") {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(OsmprjError::Io(e)),
        };

        toml::from_str(&content).map(Some).map_err(|e| OsmprjError::BadConfig {
            message: e.message().to_string(),
            src: NamedSource::new("osmprj.toml", content),
            span: e.span().map(Into::into),
        })
    }
}
