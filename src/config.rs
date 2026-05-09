use crate::error::OsmprjError;
use miette::NamedSource;
use serde::{de, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Custom usize with a minimum of 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinOneUsize(usize);

impl MinOneUsize {
    pub fn get(self) -> usize {
        self.0
    }
}

impl<'de> Deserialize<'de> for MinOneUsize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        if value < 1 {
            return Err(de::Error::custom("value must be >= 1"));
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct PostProcessConfig {
    /// Whether to run the theme's bundled SQL files after import (default: true).
    pub include_theme_sql: Option<bool>,
    /// Additional SQL file paths (relative to osmprj.toml) to run after import.
    pub extra_sql: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SourceConfig {
    pub path: Option<String>,
    pub theme: Option<String>,
    pub schema: Option<String>,
    pub srid: Option<u32>,
    pub postprocess: Option<PostProcessConfig>,
}

impl SourceConfig {
    pub fn effective_schema(&self, name: &str) -> String {
        self.schema
            .clone()
            .unwrap_or_else(|| name.replace(['/', '-'], "_"))
    }

    pub fn effective_srid(&self) -> u32 {
        self.srid.unwrap_or(3857)
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ProjectSettings {
    /// Database URL to use for importing data
    pub database_url: Option<String>,

    /// Directory where .osm.pbf files are saved to
    pub data_dir: Option<String>,

    /// Directory that all logs are written too
    pub log_dir: Option<String>,

    /// Whether we are running with an SSD
    pub ssd: Option<bool>,

    /// Max memory limit for osm2pgsql processes
    pub max_diff_size_mb: Option<u32>,

    /// Maximum number of PBF downloads to run concurrently. Defaults to 3.
    pub max_concurrent_downloads: Option<MinOneUsize>,

    /// Maximum number of osm2pgsql import pipelines to run concurrently. Defaults to 1.
    /// Each import holds one PostgreSQL connection and a share of system RAM.
    pub max_concurrent_imports: Option<MinOneUsize>,
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

    pub fn effective_max_concurrent_downloads(&self) -> usize {
        self.max_concurrent_downloads
            .map(MinOneUsize::get)
            .unwrap_or(3)
    }

    pub fn effective_max_concurrent_imports(&self) -> usize {
        self.max_concurrent_imports
            .map(MinOneUsize::get)
            .unwrap_or(1)
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

        toml::from_str(&content)
            .map(Some)
            .map_err(|e| OsmprjError::BadConfig {
                message: e.message().to_string(),
                src: NamedSource::new("osmprj.toml", content),
                span: e.span().map(Into::into),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_max_concurrent_downloads_default() {
        let s = ProjectSettings::default();
        assert_eq!(s.effective_max_concurrent_downloads(), 3);
    }

    #[test]
    fn effective_max_concurrent_downloads_configured() {
        let s = ProjectSettings {
            max_concurrent_downloads: Some(MinOneUsize(5)),
            ..Default::default()
        };
        assert_eq!(s.effective_max_concurrent_downloads(), 5);
    }

    #[test]
    fn effective_max_concurrent_imports_default() {
        let s = ProjectSettings::default();
        assert_eq!(s.effective_max_concurrent_imports(), 1);
    }

    #[test]
    fn effective_max_concurrent_imports_configured() {
        let s = ProjectSettings {
            max_concurrent_imports: Some(MinOneUsize(4)),
            ..Default::default()
        };
        assert_eq!(s.effective_max_concurrent_imports(), 4);
    }

    #[test]
    fn project_settings_rejects_zero_downloads() {
        let input = r#"
        [project]
        max_concurrent_downloads = 0
    "#;

        let parsed: Result<ProjectConfig, toml::de::Error> = toml::from_str(input);
        assert!(parsed.is_err());
    }

    #[test]
    fn project_settings_rejects_zero_imports() {
        let input = r#"
        [project]
        max_concurrent_imports = 0
    "#;

        let parsed: Result<ProjectConfig, toml::de::Error> = toml::from_str(input);
        assert!(parsed.is_err());
    }
}
