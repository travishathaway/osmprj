use crate::config::ProjectConfig;
use crate::db;

pub async fn run(config: &ProjectConfig) -> Result<(), crate::error::OsmprjError> {
    let sources = &config.sources;

    // ── Database connection ───────────────────────────────────────────────────
    let url = config.project.database_url.as_deref();

    let client = match url {
        None => {
            println!("  database:  not configured");
            println!(
                "             Add database_url to [project] in osmprj.toml to enable \
                 connection checks"
            );
            print_sources_no_db(config);
            return Ok(());
        }
        Some(u) => match db::connect(u).await {
            Ok(c) => {
                println!("  database:  {u}  ✓ connected");
                c
            }
            Err(e) => {
                println!("  database:  {u}  ✗ connection failed");
                println!("             {e}");
                println!(
                    "             Check that PostgreSQL is running and the URL is correct.\n\
                     \n             Tip: run `psql \"{u}\"` to test the connection directly."
                );
                print_sources_no_db(config);
                return Ok(());
            }
        },
    };

    // ── Sources ───────────────────────────────────────────────────────────────
    if sources.is_empty() {
        println!("\n  sources:   (none — run 'osmprj add' to register a source)");
        return Ok(());
    }

    // Compute column widths for aligned output
    let name_width = sources.keys().map(|k| k.len()).max().unwrap_or(0).max(6);
    let schema_width = sources
        .iter()
        .map(|(n, s)| s.effective_schema(n).len())
        .max()
        .unwrap_or(0)
        .max(6);

    println!("\n  {:<name_width$}  {:<schema_width$}  status", "source", "schema");
    println!("  {:-<name_width$}  {:-<schema_width$}  ------", "", "");

    let mut sorted: Vec<_> = sources.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    for (name, source) in sorted {
        let schema = source.effective_schema(name);
        let exists = db::schema_exists(&client, &schema).await.unwrap_or(false);
        let indicator = if exists { "✓" } else { "✗" };
        let note = if exists {
            String::new()
        } else {
            "  — run 'osmprj sync' to import".to_string()
        };
        println!("  {name:<name_width$}  {schema:<schema_width$}  {indicator}{note}");
    }

    Ok(())
}

fn print_sources_no_db(config: &ProjectConfig) {
    if config.sources.is_empty() {
        println!("\n  sources:   (none — run 'osmprj add' to register a source)");
        return;
    }

    let name_width = config.sources.keys().map(|k| k.len()).max().unwrap_or(0).max(6);
    let schema_width = config
        .sources
        .iter()
        .map(|(n, s)| s.effective_schema(n).len())
        .max()
        .unwrap_or(0)
        .max(6);

    println!("\n  {:<name_width$}  {:<schema_width$}  status", "source", "schema");
    println!("  {:-<name_width$}  {:-<schema_width$}  ------", "", "");

    let mut sorted: Vec<_> = config.sources.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    for (name, source) in sorted {
        let schema = source.effective_schema(name);
        println!("  {name:<name_width$}  {schema:<schema_width$}  (no database connection)");
    }
}
