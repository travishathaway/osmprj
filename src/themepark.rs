use crate::error::OsmprjError;
use crate::lock::{LockFile, ThemeparkLockEntry};
use chrono::Utc;
use flate2::read::GzDecoder;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;
use tempfile::NamedTempFile;

use crate::config::TopicsConfig;

const THEMEPARK_URL: &str =
    "https://github.com/osm2pgsql-dev/osm2pgsql-themepark/archive/refs/heads/master.tar.gz";

/// Known theme name → config filename mappings.
fn theme_config_map(theme: &str) -> Option<&'static str> {
    match theme {
        "shortbread_v1" => Some("shortbread.lua"),
        "shortbread_v1_gen" => Some("shortbread_gen.lua"),
        "basic" | "generic" => Some("generic.lua"),
        "osmcarto" => Some("osmcarto.lua"),
        "experimental" => Some("experimental.lua"),
        _ => None,
    }
}

/// Returns the root of the extracted themepark directory inside `cache_dir`.
fn themepark_root(cache_dir: &Path) -> PathBuf {
    cache_dir.join("osmprj").join("themepark")
}

/// Finds the single subdirectory created by the tarball extraction.
fn extracted_dir(themepark_dir: &Path) -> Option<PathBuf> {
    fs::read_dir(themepark_dir).ok()?.flatten().find_map(|e| {
        let p = e.path();
        if p.is_dir() { Some(p) } else { None }
    })
}

pub async fn ensure_cached(cache_dir: &Path, lock: &mut LockFile) -> Result<PathBuf, OsmprjError> {
    let themepark_dir = themepark_root(cache_dir);

    if themepark_dir.exists() {
        if let Some(root) = extracted_dir(&themepark_dir) {
            return Ok(root);
        }
    }

    println!("  Downloading osm2pgsql-themepark...");
    fs::create_dir_all(&themepark_dir).map_err(OsmprjError::Io)?;

    let response = reqwest::get(THEMEPARK_URL).await
        .map_err(|e| OsmprjError::ThemeparkDownloadFailed { message: e.to_string() })?;

    if !response.status().is_success() {
        return Err(OsmprjError::ThemeparkDownloadFailed {
            message: format!("HTTP {}", response.status()),
        });
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| OsmprjError::ThemeparkDownloadFailed { message: e.to_string() })?;

    let gz = GzDecoder::new(bytes.as_ref());
    let mut archive = Archive::new(gz);
    archive
        .unpack(&themepark_dir)
        .map_err(|e| OsmprjError::ThemeparkExtractFailed { message: e.to_string() })?;

    lock.set_themepark(ThemeparkLockEntry { cached_at: Utc::now() })?;

    extracted_dir(&themepark_dir)
        .ok_or_else(|| OsmprjError::ThemeparkExtractFailed {
            message: "No directory found after extraction".to_string(),
        })
}

pub fn resolve_config_file(themepark_root: &Path, theme: &str) -> Result<PathBuf, OsmprjError> {
    let config_dir = themepark_root.join("config");

    if let Some(filename) = theme_config_map(theme) {
        let path = config_dir.join(filename);
        if path.exists() {
            return Ok(path);
        }
    }

    // Fallback: look for <theme>.lua or <theme_normalized>.lua in config/
    let candidates = [
        format!("{theme}.lua"),
        format!("{}.lua", theme.replace('_', "-")),
        format!("{}.lua", theme.replace('-', "_")),
    ];
    for candidate in &candidates {
        let path = config_dir.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(OsmprjError::ThemeNotFound { theme: theme.to_string() })
}

/// Reads `add_topic` calls from a lua config file and returns the topic strings.
fn parse_topics_from_config(config_path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(config_path) else { return vec![] };
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            // Match: themepark:add_topic('some/topic') or themepark:add_topic("some/topic")
            if let Some(rest) = trimmed.strip_prefix("themepark:add_topic(") {
                let rest = rest.trim_end_matches(')').trim();
                let topic = rest.trim_matches(|c| c == '\'' || c == '"');
                if !topic.is_empty() {
                    return Some(topic.to_string());
                }
            }
            None
        })
        .collect()
}

pub fn generate_lua_tempfile(
    themepark_root: &Path,
    base_config: &Path,
    topics_config: &TopicsConfig,
) -> Result<NamedTempFile, OsmprjError> {
    let lua_dir = themepark_root.join("lua");
    let package_path = format!("{}/?.lua", lua_dir.display());

    let topics: Vec<String> = if let Some(ref list) = topics_config.list {
        list.clone()
    } else {
        let mut base = parse_topics_from_config(base_config);
        if let Some(ref adds) = topics_config.add {
            for t in adds {
                if !base.contains(t) {
                    base.push(t.clone());
                }
            }
        }
        if let Some(ref removes) = topics_config.remove {
            base.retain(|t| !removes.contains(t));
        }
        base
    };

    let mut lua = format!(
        "package.path = '{package_path};' .. package.path\nlocal themepark = require('themepark')\n"
    );
    for topic in &topics {
        lua.push_str(&format!("themepark:add_topic('{topic}')\n"));
    }

    let mut tmp = NamedTempFile::new().map_err(OsmprjError::Io)?;
    std::io::Write::write_all(&mut tmp, lua.as_bytes()).map_err(OsmprjError::Io)?;
    Ok(tmp)
}
