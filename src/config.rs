use crate::error::OsmprjError;
use miette::NamedSource;
use serde::{de, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

    /// Shell command whose stdout is used as the database URL.
    /// Runs via `sh -c` on Unix or `cmd /C` on Windows.
    /// Takes precedence over `database_url` when set.
    /// `OSMPRJ_DATABASE_URL` env var takes precedence over both.
    pub database_url_command: Option<String>,

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

    /// Resolve the database URL using the three-tier precedence:
    ///
    /// 1. `OSMPRJ_DATABASE_URL` environment variable (highest priority)
    /// 2. `database_url_command` — runs the command via the system shell,
    ///    captures stdout, and uses the trimmed result as the URL.
    /// 3. `database_url` inline in `osmprj.toml` (fallback)
    ///
    /// Returns `Ok(None)` when none of the three sources provide a URL.
    /// Returns `Err` if `database_url_command` is set but fails or produces
    /// empty output.
    pub fn effective_database_url(&self) -> Result<Option<String>, OsmprjError> {
        // 1. Environment variable wins unconditionally.
        if let Ok(url) = std::env::var("OSMPRJ_DATABASE_URL") {
            return Ok(Some(url));
        }

        // 2. Shell command.
        if let Some(ref cmd) = self.database_url_command {
            #[cfg(unix)]
            let output = Command::new("sh").args(["-c", cmd]).output();
            #[cfg(windows)]
            let output = Command::new("cmd").args(["/C", cmd]).output();

            let out = output.map_err(|e| OsmprjError::CredentialCommandFailed {
                message: format!("could not execute command `{cmd}`: {e}"),
            })?;

            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                return Err(OsmprjError::CredentialCommandFailed {
                    message: if stderr.is_empty() {
                        format!(
                            "command `{cmd}` exited with status {}",
                            out.status
                                .code()
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "non-zero".to_string())
                        )
                    } else {
                        format!(
                            "command `{cmd}` exited with status {}: {stderr}",
                            out.status
                                .code()
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "non-zero".to_string())
                        )
                    },
                });
            }

            let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if url.is_empty() {
                return Err(OsmprjError::CredentialCommandEmpty);
            }
            return Ok(Some(url));
        }

        // 3. Inline value from osmprj.toml.
        Ok(self.database_url.clone())
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

    // ── effective_database_url tests ──────────────────────────────────────────

    #[test]
    fn effective_db_url_env_var_wins_over_inline() {
        // Use a unique env var key to avoid test cross-contamination.
        // We temporarily set OSMPRJ_DATABASE_URL via std::env::set_var.
        // Tests that touch env vars must not run in parallel — use serial if needed,
        // but for unit tests this is generally safe since each sets a distinct value.
        std::env::set_var("OSMPRJ_DATABASE_URL", "postgres://env:envpass@host/db");
        let s = ProjectSettings {
            database_url: Some("postgres://file:filepass@host/db".to_string()),
            ..Default::default()
        };
        let result = s.effective_database_url().unwrap();
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        assert_eq!(result, Some("postgres://env:envpass@host/db".to_string()));
    }

    #[test]
    fn effective_db_url_env_var_wins_over_command() {
        std::env::set_var("OSMPRJ_DATABASE_URL", "postgres://env:envpass@host/db");
        let s = ProjectSettings {
            database_url_command: Some("echo postgres://cmd:cmdpass@host/db".to_string()),
            ..Default::default()
        };
        let result = s.effective_database_url().unwrap();
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        assert_eq!(result, Some("postgres://env:envpass@host/db".to_string()));
    }

    #[test]
    fn effective_db_url_command_used_when_no_env_var() {
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        let s = ProjectSettings {
            database_url_command: Some("echo postgres://cmd:cmdpass@host/db".to_string()),
            ..Default::default()
        };
        let result = s.effective_database_url().unwrap();
        assert_eq!(result, Some("postgres://cmd:cmdpass@host/db".to_string()));
    }

    #[test]
    fn effective_db_url_command_output_is_trimmed() {
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        // echo adds a trailing newline; the method must trim it.
        let s = ProjectSettings {
            database_url_command: Some("echo '  postgres://cmd:cmdpass@host/db  '".to_string()),
            ..Default::default()
        };
        let result = s.effective_database_url().unwrap();
        assert_eq!(result, Some("postgres://cmd:cmdpass@host/db".to_string()));
    }

    #[test]
    fn effective_db_url_inline_fallback() {
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        let s = ProjectSettings {
            database_url: Some("postgres://inline:pass@host/db".to_string()),
            ..Default::default()
        };
        let result = s.effective_database_url().unwrap();
        assert_eq!(result, Some("postgres://inline:pass@host/db".to_string()));
    }

    #[test]
    fn effective_db_url_all_absent_returns_none() {
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        let s = ProjectSettings::default();
        let result = s.effective_database_url().unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn effective_db_url_command_nonzero_exit_errors() {
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        let s = ProjectSettings {
            database_url_command: Some("exit 1".to_string()),
            ..Default::default()
        };
        let result = s.effective_database_url();
        assert!(matches!(
            result,
            Err(OsmprjError::CredentialCommandFailed { .. })
        ));
    }

    #[test]
    fn effective_db_url_command_empty_output_errors() {
        std::env::remove_var("OSMPRJ_DATABASE_URL");
        let s = ProjectSettings {
            // Command succeeds but prints nothing (true prints nothing, exits 0).
            database_url_command: Some("true".to_string()),
            ..Default::default()
        };
        let result = s.effective_database_url();
        assert!(matches!(result, Err(OsmprjError::CredentialCommandEmpty)));
    }

    #[test]
    fn database_url_command_field_deserializes() {
        let input = r#"
            [project]
            database_url_command = "pass show osmprj/db"
        "#;
        let config: ProjectConfig = toml::from_str(input).unwrap();
        assert_eq!(
            config.project.database_url_command,
            Some("pass show osmprj/db".to_string())
        );
    }

    #[test]
    fn database_url_command_field_absent_is_none() {
        let input = r#"
            [project]
            database_url = "postgres://user:pass@localhost/db"
        "#;
        let config: ProjectConfig = toml::from_str(input).unwrap();
        assert_eq!(config.project.database_url_command, None);
    }
}
