use crate::config::TopicsConfig;
use crate::error::OsmprjError;
use crate::theme_registry::ThemeEntry;
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

/// Returns `true` if `name` is a recognized built-in themepark theme.
pub fn is_builtin_theme(name: &str) -> bool {
    theme_config_map(name).is_some()
}

/// The list of built-in theme names, for display purposes.
pub fn builtin_theme_names() -> &'static [&'static str] {
    &["shortbread_v1", "shortbread_v1_gen", "basic", "generic", "osmcarto", "experimental"]
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

/// Generate a Lua wrapper for a plugin theme of type `themepark`.
///
/// For `ThemeType::Flex` plugin themes no wrapper is generated — callers should
/// use `entry.lua_path` directly as the `--style` argument to osm2pgsql.
///
/// The wrapper is identical in structure to `generate_lua_wrapper` but the entry
/// path and `package.path` extension come from the plugin's own directory rather
/// than from `THEMEPARK_PATH`.  If `THEMEPARK_PATH` is set its `lua/` directory
/// is also prepended so that themepark's own Lua modules remain available.
pub fn generate_lua_wrapper_for_plugin(
    entry: &ThemeEntry,
    topics_config: Option<&TopicsConfig>,
    schema: &str,
) -> Result<NamedTempFile, OsmprjError> {
    // Build package.path: plugin lua/ dir first, then themepark root lua/ if available.
    let plugin_lua_dir = entry.theme_dir.join("lua");
    let mut package_path_parts: Vec<String> = Vec::new();
    if plugin_lua_dir.is_dir() {
        package_path_parts.push(format!("{}/?.lua", plugin_lua_dir.display()));
    }
    // Also add themepark's lua/ dir if THEMEPARK_PATH is set, so themepark modules load.
    if let Ok(tp_root) = std::env::var("THEMEPARK_PATH") {
        let tp_lua = PathBuf::from(tp_root).join("lua");
        if tp_lua.is_dir() {
            package_path_parts.push(format!("{}/?.lua", tp_lua.display()));
        }
    }

    let package_path_prefix = if package_path_parts.is_empty() {
        String::new()
    } else {
        format!("package.path = '{}' .. ';' .. package.path\n", package_path_parts.join(";"))
    };

    let mut lua = format!(
        "{package_path_prefix}\
         local themepark = require('themepark')\n\
         themepark:set_option('schema', '{schema}')\n"
    );

    match topics_config {
        Some(tc) => {
            let topics: Vec<String> = if let Some(ref list) = tc.list {
                list.clone()
            } else {
                let mut base = parse_topics_from_config(&entry.lua_path);
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
            lua.push_str(&format!("dofile('{}')\n", entry.lua_path.display()));
        }
    }

    let mut tmp = NamedTempFile::new().map_err(OsmprjError::Io)?;
    std::io::Write::write_all(&mut tmp, lua.as_bytes()).map_err(OsmprjError::Io)?;
    Ok(tmp)
}
