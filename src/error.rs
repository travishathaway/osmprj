use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum OsmprjError {
    #[error("osmprj.toml not found in the current directory")]
    #[diagnostic(
        code(osmprj::no_project),
        help("Run `osmprj init` to create a project file")
    )]
    ProjectNotFound,

    #[error("osmprj.toml already exists")]
    #[diagnostic(
        code(osmprj::project_exists),
        help("Delete osmprj.toml first, or run from a different directory")
    )]
    ProjectExists,

    #[error("Failed to parse osmprj.toml: {message}")]
    #[diagnostic(code(osmprj::bad_config))]
    BadConfig {
        message: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("error here")]
        span: Option<SourceSpan>,
    },

    #[error("Source '{name}' already exists in osmprj.toml")]
    #[diagnostic(
        code(osmprj::duplicate_source),
        help("Remove the existing entry or choose a different name")
    )]
    DuplicateSource { name: String },

    #[error("Source '{name}' not found in osmprj.toml")]
    #[diagnostic(
        code(osmprj::source_not_found),
        help("Run `osmprj status` to list configured sources")
    )]
    SourceNotFound { name: String },

    #[error("'{id}' was not found in the Geofabrik index")]
    #[diagnostic(
        code(osmprj::unknown_geofabrik_id),
        help("Visit https://download.geofabrik.de to browse valid region IDs")
    )]
    UnknownGeofabrikId { id: String },

    #[error("--name is required when using --path")]
    #[diagnostic(
        code(osmprj::missing_name),
        help("Example: osmprj add --path region.osm.pbf --name my-region")
    )]
    MissingName,

    #[error("Provide either a Geofabrik ID or both --path and --name")]
    #[diagnostic(code(osmprj::invalid_args))]
    InvalidArgs,

    #[error("--schema cannot be used with multiple Geofabrik IDs")]
    #[diagnostic(
        code(osmprj::schema_with_multiple_ids),
        help("Omit --schema to use the auto-derived schema name for each region")
    )]
    SchemaWithMultipleIds,

    #[error("No database URL configured")]
    #[diagnostic(
        code(osmprj::no_database_url),
        help(
            "Add database_url to the [project] section in osmprj.toml:\n\
             \n  database_url = \"postgres://user:pass@localhost/dbname\""
        )
    )]
    NoDatabaseUrl,

    #[error("Could not connect to database: {message}")]
    #[diagnostic(
        code(osmprj::db_connect_failed),
        help(
            "Check that PostgreSQL is running and reachable at the configured URL.\n\
             \n  Tip: run `psql \"{url}\"` to test the connection directly."
        )
    )]
    DatabaseConnectFailed { message: String, url: String },

    #[error("Database query failed: {message}")]
    #[diagnostic(code(osmprj::db_query_failed))]
    DatabaseQueryFailed { message: String },

    #[error("Failed to fetch Geofabrik index: {message}")]
    #[diagnostic(
        code(osmprj::fetch_failed),
        help("Check your internet connection and try again")
    )]
    GeofabrikFetchFailed { message: String },

    #[error("Could not determine the OS cache directory")]
    #[diagnostic(code(osmprj::no_cache_dir))]
    NoCacheDir,

    #[error("Failed to read osmprj.lock: {message}")]
    #[diagnostic(code(osmprj::lock_read_failed))]
    LockReadFailed { message: String },

    #[error("Failed to write osmprj.lock: {message}")]
    #[diagnostic(code(osmprj::lock_write_failed))]
    LockWriteFailed { message: String },

    #[error("Unknown source(s): {names}")]
    #[diagnostic(
        code(osmprj::unknown_sources),
        help("Check that the source names match entries in osmprj.toml")
    )]
    UnknownSources { names: String },

    #[error("'{binary}' not found on PATH")]
    #[diagnostic(
        code(osmprj::binary_not_found),
        help("Install osm2pgsql and ensure it is on your PATH")
    )]
    BinaryNotFound { binary: String },

    #[error("osm2pgsql-themepark is not installed (THEMEPARK_PATH is not set or does not exist)")]
    #[diagnostic(
        code(osmprj::themepark_not_found),
        help("Install osm2pgsql-themepark and ensure THEMEPARK_PATH points to its root directory")
    )]
    ThemeparkNotFound,

    #[error("Theme '{theme}' not found in osm2pgsql-themepark config directory")]
    #[diagnostic(
        code(osmprj::theme_not_found),
        help("Check available themes in the themepark config/ directory")
    )]
    ThemeNotFound { theme: String },

    #[error("Download of '{url}' failed: {message}")]
    #[diagnostic(code(osmprj::download_failed))]
    DownloadFailed { url: String, message: String },

    #[error("MD5 verification failed for '{name}': expected {expected}, got {actual}")]
    #[diagnostic(code(osmprj::md5_mismatch))]
    Md5Mismatch { name: String, expected: String, actual: String },

    #[error("Import of '{name}' failed with exit code {code}")]
    #[diagnostic(code(osmprj::import_failed))]
    ImportFailed { name: String, code: i32 },

    #[error("osm2pgsql-replication init failed for '{name}': exit code {code}")]
    #[diagnostic(code(osmprj::replication_init_failed))]
    ReplicationInitFailed { name: String, code: i32 },

    #[error("osm2pgsql-replication update failed for '{name}': exit code {code}")]
    #[diagnostic(code(osmprj::replication_update_failed))]
    ReplicationUpdateFailed { name: String, code: i32 },

    #[error(transparent)]
    #[diagnostic(code(osmprj::io))]
    Io(#[from] std::io::Error),
}
