use crate::error::OsmprjError;
use crate::output;
use crate::theme_registry::{ThemeRegistry, ThemeType};
use console::style;

pub fn run_list() -> Result<(), OsmprjError> {
    let registry = ThemeRegistry::build();
    let plugins = registry.all();

    println!();

    if plugins.is_empty() {
        println!("  {} No plugin themes found.", output::icon_info());
        println!();
        println!("  osmprj searched:");
        for p in registry.searched_paths() {
            println!("    {}", p.display());
        }
        println!();
        println!("  Install a theme package or set OSMPRJ_THEME_PATH to add plugin themes.");
    } else {
        println!("  {}", style("PLUGIN THEMES").bold());
        println!();
        for entry in plugins {
            let type_label = match entry.theme_type() {
                ThemeType::Themepark => "themepark",
                ThemeType::Flex => "flex     ",
            };
            let sql_info = if entry.sql_files.is_empty() {
                "(no sql)".to_string()
            } else {
                entry
                    .sql_files
                    .iter()
                    .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            println!(
                "  {}  {}  \"{}\"  v{}",
                style(&entry.manifest.name).green().bold(),
                style(type_label).cyan(),
                entry.manifest.description,
                entry.manifest.version,
            );
            println!("  {}", style(entry.theme_dir.display()).dim());
            println!("  sql: {sql_info}");
            println!();
        }
    }

    Ok(())
}
