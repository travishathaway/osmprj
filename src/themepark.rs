use crate::config::TopicsConfig;
use crate::error::OsmprjError;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Returns the themepark root directory from the THEMEPARK_PATH environment variable.
pub fn find_root() -> Result<PathBuf, OsmprjError> {
    let path = std::env::var("THEMEPARK_PATH")
        .map(PathBuf::from)
        .map_err(|_| OsmprjError::ThemeparkNotFound)?;
    if !path.is_dir() {
        return Err(OsmprjError::ThemeparkNotFound);
    }
    Ok(path)
}

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

/// Generate a Lua wrapper that sets the schema and enumerates topics, then writes it to a
/// NamedTempFile. Always use this instead of the base config file directly so that
/// themepark's default `schema = 'public'` is overridden before any table is defined.
///
/// When `topics_config` is None the base config's own topic list is preserved by loading
/// it with `dofile`; Lua's module cache means the already-configured `themepark` instance
/// (with the correct schema) is reused inside the dofile'd script.
pub fn generate_lua_wrapper(
    themepark_root: &Path,
    base_config: &Path,
    topics_config: Option<&TopicsConfig>,
    schema: &str,
) -> Result<NamedTempFile, OsmprjError> {
    let lua_dir = themepark_root.join("lua");
    let package_path = format!("{}/?.lua", lua_dir.display());

    let mut lua = format!(
        "package.path = '{package_path};' .. package.path\n\
         local themepark = require('themepark')\n\
         themepark:set_option('schema', '{schema}')\n"
    );

    match topics_config {
        Some(tc) => {
            let topics: Vec<String> = if let Some(ref list) = tc.list {
                list.clone()
            } else {
                let mut base = parse_topics_from_config(base_config);
                if let Some(ref adds) = tc.add {
                    for t in adds {
                        if !base.contains(t) {
                            base.push(t.clone());
                        }
                    }
                }
                if let Some(ref removes) = tc.remove {
                    base.retain(|t| !removes.contains(t));
                }
                base
            };
            for topic in &topics {
                lua.push_str(&format!("themepark:add_topic('{topic}')\n"));
            }
        }
        None => {
            lua.push_str(&format!("dofile('{}')\n", base_config.display()));
        }
    }

    let mut tmp = NamedTempFile::new().map_err(OsmprjError::Io)?;
    std::io::Write::write_all(&mut tmp, lua.as_bytes()).map_err(OsmprjError::Io)?;
    Ok(tmp)
}
