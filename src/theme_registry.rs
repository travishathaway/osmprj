use serde::Deserialize;
use std::path::{Path, PathBuf};

// ─── Theme type ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeType {
    Themepark,
    Flex,
}

// ─── Manifest (theme.toml) ────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(rename = "type")]
    pub theme_type: ThemeType,
    /// Relative path to the Lua entry point from the theme directory.
    pub entry: String,
}

// ─── Theme entry (resolved) ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub manifest: ThemeManifest,
    /// Absolute path to the theme directory.
    pub theme_dir: PathBuf,
    /// Absolute path to the Lua entry point.
    pub lua_path: PathBuf,
    /// Sorted list of SQL post-processing files from <theme_dir>/sql/.
    pub sql_files: Vec<PathBuf>,
}

impl ThemeEntry {
    pub fn theme_type(&self) -> &ThemeType {
        &self.manifest.theme_type
    }
}

// ─── Registry ─────────────────────────────────────────────────────────────────

pub struct ThemeRegistry {
    entries: Vec<ThemeEntry>,
    searched_paths: Vec<PathBuf>,
}

impl ThemeRegistry {
    /// Collect all paths to scan in priority order (highest first).
    ///
    /// Tier 1: OSMPRJ_THEME_PATH env var (colon/semicolon separated)
    /// Tier 2: <exe_prefix>/share/osmprj/themes/  (exe-relative, works for any package manager)
    /// Tier 3: dirs::data_dir()/osmprj/themes/    (user-local)
    pub fn search_paths() -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();

        // Tier 1: env var override
        if let Ok(val) = std::env::var("OSMPRJ_THEME_PATH") {
            for p in std::env::split_paths(&val) {
                paths.push(p);
            }
        }

        // Tier 2: exe-relative system path
        if let Ok(exe) = std::env::current_exe() {
            if let Some(prefix) = exe.parent().and_then(|bin| bin.parent()) {
                paths.push(prefix.join("share").join("osmprj").join("themes"));
            }
        }

        // Tier 3: user-local data dir
        if let Some(data_dir) = dirs::data_dir() {
            paths.push(data_dir.join("osmprj").join("themes"));
        }

        paths
    }

    /// Scan all search paths and build the theme registry.
    ///
    /// - Non-existent directories are silently skipped.
    /// - Subdirectories without a valid `theme.toml` are silently skipped.
    /// - Malformed `theme.toml` files emit a warning and are skipped.
    /// - When two paths provide a theme with the same name, the higher-priority
    ///   (earlier) tier wins.
    pub fn build() -> Self {
        let searched_paths = Self::search_paths();
        let mut entries: Vec<ThemeEntry> = Vec::new();

        for search_path in &searched_paths {
            if !search_path.is_dir() {
                continue;
            }

            let read_dir = match std::fs::read_dir(search_path) {
                Ok(rd) => rd,
                Err(_) => continue,
            };

            for dir_entry in read_dir.flatten() {
                let theme_dir = dir_entry.path();
                if !theme_dir.is_dir() {
                    continue;
                }

                let manifest_path = theme_dir.join("theme.toml");
                if !manifest_path.exists() {
                    continue;
                }

                let manifest = match Self::load_manifest(&manifest_path) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!(
                            "  warning: skipping theme at '{}': {e}",
                            theme_dir.display()
                        );
                        continue;
                    }
                };

                // Higher-priority tier wins: skip if we already have this name.
                if entries.iter().any(|e| e.manifest.name == manifest.name) {
                    continue;
                }

                let lua_path = theme_dir.join(&manifest.entry);
                let sql_files = Self::collect_sql_files(&theme_dir);

                entries.push(ThemeEntry {
                    manifest,
                    theme_dir,
                    lua_path,
                    sql_files,
                });
            }
        }

        ThemeRegistry {
            entries,
            searched_paths,
        }
    }

    fn load_manifest(path: &Path) -> Result<ThemeManifest, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("cannot read theme.toml: {e}"))?;
        toml::from_str(&content).map_err(|e| format!("invalid theme.toml: {e}"))
    }

    fn collect_sql_files(theme_dir: &Path) -> Vec<PathBuf> {
        let sql_dir = theme_dir.join("sql");
        if !sql_dir.is_dir() {
            return vec![];
        }
        let mut files: Vec<PathBuf> = std::fs::read_dir(&sql_dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("sql"))
            .collect();
        files.sort();
        files
    }

    /// Look up a theme by name. Returns the first match (highest-priority tier).
    pub fn find(&self, name: &str) -> Option<&ThemeEntry> {
        self.entries.iter().find(|e| e.manifest.name == name)
    }

    /// All discovered theme entries.
    pub fn all(&self) -> &[ThemeEntry] {
        &self.entries
    }

    /// All paths that were checked during discovery (for error diagnostics).
    pub fn searched_paths(&self) -> &[PathBuf] {
        &self.searched_paths
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_theme(dir: &Path, name: &str, theme_type: &str, with_sql: bool) {
        let theme_dir = dir.join(name);
        fs::create_dir_all(&theme_dir).unwrap();
        fs::write(
            theme_dir.join("theme.toml"),
            format!(
                r#"name = "{name}"
version = "1.0.0"
description = "Test theme {name}"
type = "{theme_type}"
entry = "{name}.lua"
"#
            ),
        )
        .unwrap();
        fs::write(theme_dir.join(format!("{name}.lua")), "-- stub").unwrap();
        if with_sql {
            let sql_dir = theme_dir.join("sql");
            fs::create_dir_all(&sql_dir).unwrap();
            fs::write(
                sql_dir.join("01_indexes.sql"),
                "CREATE INDEX ON {schema}.foo(id);",
            )
            .unwrap();
        }
    }

    #[test]
    fn test_discovers_themepark_and_flex_themes() {
        let tmp = TempDir::new().unwrap();
        make_theme(tmp.path(), "tp-theme", "themepark", true);
        make_theme(tmp.path(), "flex-theme", "flex", false);

        // Use OSMPRJ_THEME_PATH to point at our temp dir.
        // Use a unique env var name per test run via a sub-scope to avoid
        // parallel test interference (best effort; tests are single-threaded by default).
        let path_val = std::env::join_paths([tmp.path()]).unwrap();
        std::env::set_var("OSMPRJ_THEME_PATH", &path_val);

        let registry = ThemeRegistry::build();

        std::env::remove_var("OSMPRJ_THEME_PATH");

        let tp = registry
            .find("tp-theme")
            .expect("themepark theme not found");
        assert_eq!(tp.manifest.theme_type, ThemeType::Themepark);
        assert_eq!(tp.sql_files.len(), 1);

        let fl = registry.find("flex-theme").expect("flex theme not found");
        assert_eq!(fl.manifest.theme_type, ThemeType::Flex);
        assert!(fl.sql_files.is_empty());
    }

    #[test]
    fn test_priority_shadowing() {
        let high = TempDir::new().unwrap();
        let low = TempDir::new().unwrap();

        make_theme(high.path(), "my-theme", "themepark", false);
        make_theme(low.path(), "my-theme", "flex", false);

        // high comes first → wins
        let path_val = std::env::join_paths([high.path(), low.path()]).unwrap();
        std::env::set_var("OSMPRJ_THEME_PATH", &path_val);

        let registry = ThemeRegistry::build();

        std::env::remove_var("OSMPRJ_THEME_PATH");

        let entry = registry.find("my-theme").expect("theme not found");
        // Should come from high-priority dir (themepark type)
        assert_eq!(entry.manifest.theme_type, ThemeType::Themepark);
        assert_eq!(entry.theme_dir.parent().unwrap(), high.path());
    }

    #[test]
    fn test_malformed_manifest_skipped() {
        let tmp = TempDir::new().unwrap();
        let bad_dir = tmp.path().join("bad-theme");
        fs::create_dir_all(&bad_dir).unwrap();
        // Missing required fields
        fs::write(bad_dir.join("theme.toml"), "name = \"bad-theme\"\n").unwrap();

        make_theme(tmp.path(), "good-theme", "flex", false);

        let path_val = std::env::join_paths([tmp.path()]).unwrap();
        std::env::set_var("OSMPRJ_THEME_PATH", &path_val);

        let registry = ThemeRegistry::build();

        std::env::remove_var("OSMPRJ_THEME_PATH");

        // Bad theme was skipped, good theme is present
        assert!(registry.find("bad-theme").is_none());
        assert!(registry.find("good-theme").is_some());
    }
}
