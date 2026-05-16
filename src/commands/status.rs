use crate::config::ProjectConfig;
use crate::db;
use crate::output;
use crate::url_utils::mask_db_url;

pub async fn run(config: &ProjectConfig) -> Result<(), crate::error::OsmprjError> {
    let sources = &config.sources;

    // ── Database connection ───────────────────────────────────────────────────
    let url = config.project.effective_database_url()?;
    let url = url.as_deref();

    let mut client = match url {
        None => {
            println!("  database:  not configured");
            println!(
                "             Add database_url to [project] in osmprj.toml,\n\
                 \n             or set the OSMPRJ_DATABASE_URL environment variable."
            );
            print_sources_no_db(config);
            return Ok(());
        }
        Some(u) => match db::connect(u).await {
            Ok(c) => {
                println!(
                    "  database:  {}  {} connected",
                    mask_db_url(u),
                    output::icon_success()
                );
                c
            }
            Err(e) => {
                println!(
                    "  database:  {}  {} connection failed",
                    mask_db_url(u),
                    output::icon_error()
                );
                println!("             {e}");
                println!(
                    "             Check that PostgreSQL is running and verify your database credentials."
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

    println!(
        "\n  {:<name_width$}  {:<schema_width$}  status",
        "source", "schema"
    );
    println!("  {:-<name_width$}  {:-<schema_width$}  ------", "", "");

    let mut sorted: Vec<_> = sources.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    for (name, source) in sorted {
        let schema = source.effective_schema(name);
        let exists = db::schema_exists(&mut client, &schema)
            .await
            .unwrap_or(false);
        let indicator = if exists {
            output::icon_success().to_string()
        } else {
            output::icon_error().to_string()
        };
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

    let name_width = config
        .sources
        .keys()
        .map(|k| k.len())
        .max()
        .unwrap_or(0)
        .max(6);
    let schema_width = config
        .sources
        .iter()
        .map(|(n, s)| s.effective_schema(n).len())
        .max()
        .unwrap_or(0)
        .max(6);

    println!(
        "\n  {:<name_width$}  {:<schema_width$}  status",
        "source", "schema"
    );
    println!("  {:-<name_width$}  {:-<schema_width$}  ------", "", "");

    let mut sorted: Vec<_> = config.sources.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    for (name, source) in sorted {
        let schema = source.effective_schema(name);
        println!("  {name:<name_width$}  {schema:<schema_width$}  (no database connection)");
    }
}
