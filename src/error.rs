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

    #[error("Failed to fetch Geofabrik index: {message}")]
    #[diagnostic(
        code(osmprj::fetch_failed),
        help("Check your internet connection and try again")
    )]
    GeofabrikFetchFailed { message: String },

    #[error("Could not determine the OS cache directory")]
    #[diagnostic(code(osmprj::no_cache_dir))]
    NoCacheDir,

    #[error(transparent)]
    #[diagnostic(code(osmprj::io))]
    Io(#[from] std::io::Error),
}
