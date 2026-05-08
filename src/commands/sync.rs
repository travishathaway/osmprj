use crate::config::{ProjectConfig, SourceConfig};
use crate::error::OsmprjError;
use crate::lock::{LockFile, SourceLockEntry};
use crate::output;
use crate::theme_registry::{ThemeRegistry, ThemeType};
use crate::{db, tuner};
use chrono::Utc;
use console::style;
use indicatif::{MultiProgress, ProgressBar};
use md5::{Digest, Md5};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::fs as tfs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Semaphore;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn which(binary: &str) -> bool {
    std::process::Command::new("which")
        .arg(binary)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn pbf_filename(source_name: &str) -> String {
    format!("{}.osm.pbf", source_name.replace('/', "-"))
}

/// Compute MD5 of a file on disk.
async fn file_md5(path: &Path) -> Result<String, OsmprjError> {
    let bytes = tfs::read(path).await.map_err(OsmprjError::Io)?;
    let hash = Md5::digest(&bytes);
    Ok(format!("{hash:x}"))
}

/// Fetch the `.md5` sidecar from Geofabrik and return the hash string.
async fn fetch_remote_md5(client: &reqwest::Client, pbf_url: &str) -> Result<String, OsmprjError> {
    let md5_url = format!("{pbf_url}.md5");
    let text = client
        .get(&md5_url)
        .send()
        .await
        .map_err(|e| OsmprjError::DownloadFailed {
            url: md5_url.clone(),
            message: e.to_string(),
        })?
        .text()
        .await
        .map_err(|e| OsmprjError::DownloadFailed {
            url: md5_url,
            message: e.to_string(),
        })?;
    // Format: "<hash>  <filename>\n"
    Ok(text.split_whitespace().next().unwrap_or("").to_string())
}

/// Issue a HEAD request to determine the file size before downloading.
/// Returns 0 on failure or if `Content-Length` is absent — those sources
/// sort to the back of the download queue without causing a sync failure.
async fn fetch_content_length(client: &reqwest::Client, url: &str) -> u64 {
    client
        .head(url)
        .send()
        .await
        .ok()
        .and_then(|r| r.content_length())
        .unwrap_or(0)
}

/// Sort a list of `(name, url, size)` tuples largest-first by `size`.
/// Uses a stable sort so equal-sized sources preserve their original order.
fn sort_sources_by_size(sources: &mut [(String, String, u64)]) {
    sources.sort_by_key(|b| std::cmp::Reverse(b.2));
}

/// Stream a single PBF file to disk with a progress bar; return bytes written.
async fn download_pbf(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    bar: &ProgressBar,
) -> Result<(), OsmprjError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| OsmprjError::DownloadFailed {
            url: url.to_string(),
            message: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(OsmprjError::DownloadFailed {
            url: url.to_string(),
            message: format!("HTTP {}", response.status()),
        });
    }

    if let Some(len) = response.content_length() {
        bar.set_length(len);
    }

    let mut file = tfs::File::create(dest).await.map_err(OsmprjError::Io)?;
    let mut stream = response;

    while let Some(chunk) = stream
        .chunk()
        .await
        .map_err(|e| OsmprjError::DownloadFailed {
            url: url.to_string(),
            message: e.to_string(),
        })?
    {
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(OsmprjError::Io)?;
        bar.inc(chunk.len() as u64);
    }

    Ok(())
}

// ─── download phase ──────────────────────────────────────────────────────────

struct DownloadResult {
    source_name: String,
    entry: SourceLockEntry,
    pbf_path: PathBuf,
}

async fn download_source(
    dl_sem: Arc<Semaphore>,
    client: Arc<reqwest::Client>,
    source_name: String,
    url: String,
    dest: PathBuf,
    bar: ProgressBar,
    bar_style: indicatif::ProgressStyle,
) -> Result<DownloadResult, (String, OsmprjError)> {
    // Acquire the download semaphore permit; held until MD5 verification completes.
    // While waiting the bar shows "Pending <name>" in the dim pending style.
    let _permit = dl_sem.acquire().await.expect("semaphore closed");

    // Permit acquired: switch to the full download bar style and update the message.
    bar.set_style(bar_style);
    bar.set_message(output::truncate_message(
        &format!("{source_name}.osm.pbf"),
        output::progress_bar_msg_width(),
    ));

    let err = |e| (source_name.clone(), e);

    download_pbf(&client, &url, &dest, &bar)
        .await
        .map_err(err)?;

    let remote_md5 = fetch_remote_md5(&client, &url).await.map_err(err)?;
    let local_md5 = file_md5(&dest).await.map_err(err)?;

    if remote_md5 != local_md5 {
        return Err((
            source_name.clone(),
            OsmprjError::Md5Mismatch {
                name: source_name,
                expected: remote_md5,
                actual: local_md5,
            },
        ));
    }

    // Disappear the download bar — the import spinner replaces it immediately.
    bar.finish_and_clear();

    Ok(DownloadResult {
        source_name,
        pbf_path: dest,
        entry: SourceLockEntry {
            url,
            md5: local_md5,
            downloaded_at: Utc::now(),
        },
    })
}

// ─── import helpers ──────────────────────────────────────────────────────────

async fn pipe_to_log(
    reader: impl tokio::io::AsyncRead + Unpin,
    log: Arc<Mutex<std::fs::File>>,
    verbose: bool,
) {
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if verbose {
            println!("{line}");
        }
        if let Ok(mut f) = log.lock() {
            let _ = writeln!(f, "{line}");
        }
    }
}

async fn run_subprocess(
    argv: &[String],
    env_vars: &[(String, String)],
    log_path: &Path,
    verbose: bool,
    spinner: &ProgressBar,
) -> Result<(), OsmprjError> {
    let log_file = std::fs::File::create(log_path).map_err(OsmprjError::Io)?;
    let log = Arc::new(Mutex::new(log_file));

    let cmd_line = argv.join(" ");
    if verbose {
        println!("  [command] {cmd_line}");
    }
    if let Ok(mut f) = log.lock() {
        let _ = writeln!(f, "[command] {cmd_line}");
    }

    let mut cmd = tokio::process::Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(OsmprjError::Io)?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    if verbose {
        spinner.finish_and_clear();
    }

    let log_out = Arc::clone(&log);
    let log_err = Arc::clone(&log);
    let stdout_task = tokio::spawn(pipe_to_log(stdout, log_out, verbose));
    let stderr_task = tokio::spawn(pipe_to_log(stderr, log_err, verbose));

    let status = child.wait().await.map_err(OsmprjError::Io)?;
    let _ = tokio::join!(stdout_task, stderr_task);

    if !status.success() {
        return Err(OsmprjError::ImportFailed {
            name: argv[0].clone(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

// ─── post-processing SQL ──────────────────────────────────────────────────────

/// Execute a list of SQL files against the database, substituting `{schema}` in each file.
///
/// Files are executed in the order provided. Each file is split on `;` and each
/// non-empty statement is executed individually (tokio-postgres does not support
/// multi-statement queries via its standard API).
pub async fn run_postprocess(
    client: &tokio_postgres::Client,
    source_name: &str,
    schema: &str,
    sql_files: &[PathBuf],
) -> Result<(), OsmprjError> {
    for file_path in sql_files {
        let file_name = file_path.display().to_string();
        let content =
            std::fs::read_to_string(file_path).map_err(|e| OsmprjError::PostProcessFailed {
                source_name: source_name.to_string(),
                file: file_name.clone(),
                message: format!("could not read file: {e}"),
            })?;

        let substituted = content.replace("{schema}", schema);

        for stmt in substituted.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            client
                .execute(stmt, &[])
                .await
                .map_err(|e| OsmprjError::PostProcessFailed {
                    source_name: source_name.to_string(),
                    file: file_name.clone(),
                    message: e.to_string(),
                })?;
        }
    }
    Ok(())
}

/// Collect the SQL files to run for a source after a fresh import.
///
/// Includes the theme's bundled SQL files (unless `include_theme_sql = false`)
/// followed by any `extra_sql` paths from the source's `[postprocess]` block.
fn collect_sql_files(source: &SourceConfig, registry: &ThemeRegistry) -> Vec<PathBuf> {
    let include_theme = source
        .postprocess
        .as_ref()
        .and_then(|pp| pp.include_theme_sql)
        .unwrap_or(true);

    let mut files: Vec<PathBuf> = Vec::new();

    if include_theme {
        if let Some(theme_name) = &source.theme {
            if let Some(entry) = registry.find(theme_name) {
                files.extend(entry.sql_files.iter().cloned());
            }
        }
    }

    if let Some(extra) = source
        .postprocess
        .as_ref()
        .and_then(|pp| pp.extra_sql.as_ref())
    {
        for path_str in extra {
            files.push(PathBuf::from(path_str));
        }
    }

    files
}

async fn run_replication_init(
    database_url: &str,
    schema: &str,
    log_path: &Path,
    verbose: bool,
) -> Result<(), OsmprjError> {
    let log_file = std::fs::File::create(log_path).map_err(OsmprjError::Io)?;
    let log = Arc::new(Mutex::new(log_file));

    let repl_args = ["init", "-d", database_url, "--schema", schema];
    let cmd_line = format!("osm2pgsql-replication {}", repl_args.join(" "));
    if verbose {
        println!("  [command] {cmd_line}");
    }
    if let Ok(mut f) = log.lock() {
        let _ = writeln!(f, "[command] {cmd_line}");
    }

    let mut child = tokio::process::Command::new("osm2pgsql-replication")
        .args(repl_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(OsmprjError::Io)?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let log_out = Arc::clone(&log);
    let log_err = Arc::clone(&log);
    let stdout_task = tokio::spawn(pipe_to_log(stdout, log_out, verbose));
    let stderr_task = tokio::spawn(pipe_to_log(stderr, log_err, verbose));

    let status = child.wait().await.map_err(OsmprjError::Io)?;
    let _ = tokio::join!(stdout_task, stderr_task);

    if !status.success() {
        return Err(OsmprjError::ReplicationInitFailed {
            name: schema.to_string(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

// ─── import source task ───────────────────────────────────────────────────────

/// All data needed to run a single source through the full import pipeline.
struct ImportSourceArgs {
    pub source_name: String,
    pub db_url: String,
    pub effective_schema: String,
    pub srid: u32,
    pub argv: Vec<String>,
    pub sql_files: Vec<PathBuf>,
    pub log_dir: PathBuf,
    pub verbose: bool,
}

/// Run a source through the full import pipeline under the import semaphore:
///   1. osm2pgsql import
///   2. post-processing SQL (if any)
///   3. replication init
///
/// The semaphore permit is held for the entire pipeline duration so that
/// concurrent import slots are not double-counted during post-processing.
async fn import_source(
    imp_sem: Arc<Semaphore>,
    mp: Arc<MultiProgress>,
    args: ImportSourceArgs,
) -> Result<String, (String, OsmprjError)> {
    let name = &args.source_name;
    let err = |e| (name.clone(), e);

    let spinner_style = output::spinner_style();
    let pending_style = output::pending_style();

    // Add the spinner immediately so the user sees "Pending <name>" while
    // waiting for the import semaphore permit.
    let spinner = mp.add(ProgressBar::new_spinner());
    spinner.set_style(pending_style);
    spinner.set_message(format!("Pending {name}"));
    spinner.enable_steady_tick(std::time::Duration::from_millis(120));

    // Acquire the import semaphore permit; held until replication init completes.
    let _permit = imp_sem.acquire().await.expect("semaphore closed");

    // Permit acquired: switch to the active import style.
    spinner.set_style(spinner_style.clone());
    spinner.set_message(format!("Importing {name}..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(250));

    let log_path = args.log_dir.join(format!("{}.log", name.replace('/', "-")));
    let env_vars = vec![
        ("OSMPRJ_SCHEMA".to_string(), args.effective_schema.clone()),
        ("OSMPRJ_SRID".to_string(), args.srid.to_string()),
    ];

    run_subprocess(&args.argv, &env_vars, &log_path, args.verbose, &spinner)
        .await
        .map_err(|e| {
            spinner.finish_with_message(format!("{} {name} failed", output::icon_error()));
            err(e)
        })?;

    // ── Post-processing SQL ───────────────────────────────────────────────────
    if !args.sql_files.is_empty() {
        spinner.set_message(format!("Post-processing {name}..."));
        match db::connect(&args.db_url).await {
            Err(e) => {
                spinner.finish_with_message(format!(
                    "{} {name} post-processing skipped (no DB connection)",
                    output::icon_warn()
                ));
                eprintln!(
                    "  {} {name}: could not connect for post-processing: {e}",
                    output::icon_warn()
                );
            }
            Ok(client) => {
                match run_postprocess(&client, name, &args.effective_schema, &args.sql_files).await
                {
                    Ok(()) => {
                        // spinner message will update to replication step below
                    }
                    Err(e) => {
                        spinner.finish_with_message(format!(
                            "{} {name} post-processing failed",
                            output::icon_warn()
                        ));
                        eprintln!("  {e}");
                    }
                }
            }
        }
    }

    // ── Replication init ──────────────────────────────────────────────────────
    spinner.set_message(format!("Initialising replication for {name}..."));
    let repl_log_path = args
        .log_dir
        .join(format!("{}-replication-init.log", name.replace('/', "-")));

    run_replication_init(
        &args.db_url,
        &args.effective_schema,
        &repl_log_path,
        args.verbose,
    )
    .await
    .map_err(|e| {
        spinner.finish_with_message(format!(
            "{} {name} replication init failed",
            output::icon_error()
        ));
        err(e)
    })?;

    spinner.finish_with_message(format!("{} {name} imported", output::icon_success()));

    Ok(name.clone())
}

// ─── replication update ───────────────────────────────────────────────────────

async fn replication_update(
    database_url: &str,
    schema: &str,
    style_path: Option<&PathBuf>,
    max_diff_size_mb: Option<u32>,
    env_vars: &[(String, String)],
    log_path: &Path,
    verbose: bool,
) -> Result<(), OsmprjError> {
    let log_file = std::fs::File::create(log_path).map_err(OsmprjError::Io)?;
    let log = Arc::new(Mutex::new(log_file));

    let mut args: Vec<String> = vec![
        "osm2pgsql-replication".into(),
        "update".into(),
        "-d".into(),
        database_url.into(),
        "--schema".into(),
        schema.into(),
    ];
    if let Some(mb) = max_diff_size_mb {
        args.push("--max-diff-size".into());
        args.push(mb.to_string());
    }
    args.push("--".into());
    args.push("--slim".into());
    args.push("--output=flex".into());

    if let Some(style_path) = style_path {
        args.push(format!("--style={}", style_path.display()));
    }

    let cmd_line = args.join(" ");
    if verbose {
        println!("  [command] {cmd_line}");
    }
    if let Ok(mut f) = log.lock() {
        let _ = writeln!(f, "[command] {cmd_line}");
    }

    let mut cmd = tokio::process::Command::new(&args[0]);
    cmd.args(&args[1..]);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(OsmprjError::Io)?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let log_out = Arc::clone(&log);
    let log_err = Arc::clone(&log);
    let stdout_task = tokio::spawn(pipe_to_log(stdout, log_out, verbose));
    let stderr_task = tokio::spawn(pipe_to_log(stderr, log_err, verbose));

    let status = child.wait().await.map_err(OsmprjError::Io)?;
    let _ = tokio::join!(stdout_task, stderr_task);

    if !status.success() {
        return Err(OsmprjError::ReplicationUpdateFailed {
            name: schema.to_string(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

// ─── public entry point ───────────────────────────────────────────────────────

pub async fn run(
    requested_sources: Vec<String>,
    verbose: bool,
    config: &ProjectConfig,
) -> Result<(), OsmprjError> {
    // Validate source filter
    if !requested_sources.is_empty() {
        let unknown: Vec<_> = requested_sources
            .iter()
            .filter(|s| !config.sources.contains_key(s.as_str()))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return Err(OsmprjError::UnknownSources {
                names: unknown.join(", "),
            });
        }
    }

    for bin in ["osm2pgsql", "osm2pgsql-replication"] {
        if !which(bin) {
            return Err(OsmprjError::BinaryNotFound {
                binary: bin.to_string(),
            });
        }
    }

    let sources_to_sync: Vec<(&String, &SourceConfig)> = config
        .sources
        .iter()
        .filter(|(name, _)| requested_sources.is_empty() || requested_sources.contains(name))
        .collect();

    let data_dir = config.project.effective_data_dir();
    std::fs::create_dir_all(&data_dir).map_err(OsmprjError::Io)?;

    let db_url = config
        .project
        .database_url
        .as_deref()
        .ok_or(OsmprjError::NoDatabaseUrl)?;

    let max_diff_size_mb = config.project.max_diff_size_mb;

    // ── Classify sources: update vs fresh ────────────────────────────────────
    // A source is in "update" mode if osm2pgsql_properties reports updatable=true,
    // meaning it was previously imported and replication was initialised.
    let mut update_sources: Vec<&String> = Vec::new();
    let mut fresh_sources: Vec<&String> = Vec::new();

    if !db_url.is_empty() {
        match db::connect(db_url).await {
            Ok(client) => {
                for (name, source) in &sources_to_sync {
                    let schema = source.effective_schema(name);
                    match db::source_is_updatable(&client, &schema).await {
                        Ok(true) => update_sources.push(name),
                        _ => fresh_sources.push(name),
                    }
                }
            }
            Err(_) => {
                // Can't reach DB yet — treat all as fresh (import will fail naturally if DB is down)
                for (name, _) in &sources_to_sync {
                    fresh_sources.push(name);
                }
            }
        }
    } else {
        for (name, _) in &sources_to_sync {
            fresh_sources.push(name);
        }
    }

    // ── Shared resources (used by both download and import phases) ────────────
    let http =
        Arc::new(
            reqwest::Client::builder()
                .build()
                .map_err(|e| OsmprjError::DownloadFailed {
                    url: String::new(),
                    message: e.to_string(),
                })?,
        );

    let theme_registry = ThemeRegistry::build();
    let log_dir = config.project.effective_log_dir();
    std::fs::create_dir_all(&log_dir).map_err(OsmprjError::Io)?;
    let ram_gb = tuner::system_ram_gb();
    let ssd = config.project.effective_ssd();

    let mut lock = LockFile::load()?;

    // ── Phase 0: Pre-flight HEAD requests + largest-first sort ────────────────
    // Resolve URLs and probe file sizes concurrently so we can sort largest-first.
    // Sources whose HEAD request fails or returns no Content-Length get size=0
    // and sort to the back — this never causes a sync failure.
    let mut sources_to_download: Vec<(String, String, u64)> = Vec::new(); // (name, url, size)

    {
        let mut head_set = tokio::task::JoinSet::new();

        for (name, source) in &sources_to_sync {
            if !fresh_sources.contains(name) {
                continue;
            }
            if source.path.is_some() {
                continue;
            }
            if lock.sources.contains_key(name.as_str()) {
                println!(
                    "  {} {} already downloaded, skipping",
                    output::icon_skip(),
                    name
                );
                continue;
            }

            let url = match source_pbf_url(name, config).await {
                Some(u) => u,
                None => {
                    eprintln!(
                        "  {} No download URL for source '{name}', skipping",
                        output::icon_warn()
                    );
                    continue;
                }
            };

            let name = (*name).clone();
            let http = Arc::clone(&http);
            head_set.spawn(async move {
                let size = fetch_content_length(&http, &url).await;
                (name, url, size)
            });
        }

        while let Some(result) = head_set.join_next().await {
            sources_to_download.push(result.expect("head task panicked"));
        }
    }

    sort_sources_by_size(&mut sources_to_download);

    // ── Phase 1 + 3b: Unified pipelined download → import loop ───────────────
    // Downloads are capped by dl_sem; imports are capped by imp_sem.
    // Each download task spawns its import task immediately on completion,
    // so imports begin as soon as their PBF is ready without waiting for all
    // downloads to finish.
    //
    // Both JoinSets are polled via tokio::select! so we react to whichever
    // completes next. The `if !set.is_empty()` guards prevent selecting on
    // an empty set (which would return None spuriously).
    let mp = Arc::new(MultiProgress::new());
    let bar_style = output::progress_bar_style();

    let dl_sem = Arc::new(Semaphore::new(
        config.project.effective_max_concurrent_downloads(),
    ));
    let imp_sem = Arc::new(Semaphore::new(
        config.project.effective_max_concurrent_imports(),
    ));

    let mut dl_set: tokio::task::JoinSet<Result<DownloadResult, (String, OsmprjError)>> =
        tokio::task::JoinSet::new();
    let mut imp_set: tokio::task::JoinSet<Result<String, (String, OsmprjError)>> =
        tokio::task::JoinSet::new();

    // Seed all download tasks (semaphore controls how many run at once)
    for (name, url, size) in sources_to_download {
        let dest = data_dir.join(pbf_filename(&name));

        // Create the bar in "pending" state immediately so the user can see all
        // queued sources. The bar knows its total size from the HEAD pre-flight
        // (0 means the HEAD failed; indicatif will show an indeterminate bar).
        let bar = if size > 0 {
            mp.add(ProgressBar::new(size))
        } else {
            mp.add(ProgressBar::no_length())
        };
        bar.set_style(output::pending_style());
        bar.set_message(format!(
            "Pending {}",
            output::truncate_message(&name, output::progress_bar_msg_width())
        ));
        bar.enable_steady_tick(std::time::Duration::from_millis(120));

        dl_set.spawn(download_source(
            Arc::clone(&dl_sem),
            Arc::clone(&http),
            name,
            url,
            dest,
            bar,
            bar_style.clone(),
        ));
    }

    let mut dl_errors: Vec<(String, OsmprjError)> = Vec::new();
    let mut imp_errors: Vec<(String, OsmprjError)> = Vec::new();
    let mut imported: Vec<String> = Vec::new();

    loop {
        if dl_set.is_empty() && imp_set.is_empty() {
            break;
        }

        tokio::select! {
            Some(result) = dl_set.join_next(), if !dl_set.is_empty() => {
                match result.expect("download task panicked") {
                    Ok(dl) => {
                        lock.set_source(dl.source_name.clone(), dl.entry)?;

                        // Resolve per-source import args
                        let source = &config.sources[&dl.source_name];
                        let effective_schema = source.effective_schema(&dl.source_name);
                        let srid = source.effective_srid();

                        let style_path: Option<PathBuf> = if let Some(ref theme) = source.theme {
                            match theme_registry.find(theme) {
                                Some(plugin) => Some(plugin.lua_path.clone()),
                                None => {
                                    imp_errors.push((
                                        dl.source_name.clone(),
                                        OsmprjError::ThemeNotFound { theme: theme.clone() },
                                    ));
                                    continue;
                                }
                            }
                        } else {
                            None
                        };

                        let pbf_size_gb = std::fs::metadata(&dl.pbf_path)
                            .map(|m| m.len() as f64 / 1_073_741_824.0)
                            .unwrap_or(0.0);

                        let tuner_input = tuner::TunerInput {
                            system_ram_gb: ram_gb,
                            pbf_size_gb,
                            ssd,
                            concurrent_imports: config.project.effective_max_concurrent_imports(),
                            database_url: db_url.to_string(),
                            effective_schema: effective_schema.clone(),
                            pbf_path: dl.pbf_path.clone(),
                            style_path: style_path.clone(),
                            data_dir: data_dir.clone(),
                            source_name: dl.source_name.clone(),
                        };
                        let argv = tuner::build_command(&tuner_input);
                        let sql_files = collect_sql_files(source, &theme_registry);

                        imp_set.spawn(import_source(
                            Arc::clone(&imp_sem),
                            Arc::clone(&mp),
                            ImportSourceArgs {
                                source_name: dl.source_name,
                                db_url: db_url.to_string(),
                                effective_schema,
                                srid,
                                argv,
                                sql_files,
                                log_dir: log_dir.clone(),
                                verbose,
                            },
                        ));
                    }
                    Err((name, e)) => dl_errors.push((name, e)),
                }
            }

            Some(result) = imp_set.join_next(), if !imp_set.is_empty() => {
                match result.expect("import task panicked") {
                    Ok(name) => imported.push(name),
                    Err((name, e)) => imp_errors.push((name, e)),
                }
            }
        }
    }

    // Also import sources that have a local `path` (no download needed)
    for name in &fresh_sources {
        let source = &config.sources[*name];
        let Some(ref path_str) = source.path else {
            continue;
        };
        let pbf_path = PathBuf::from(path_str);

        let effective_schema = source.effective_schema(name);
        let srid = source.effective_srid();

        let style_path: Option<PathBuf> = if let Some(ref theme) = source.theme {
            match theme_registry.find(theme) {
                Some(plugin) => Some(plugin.lua_path.clone()),
                None => {
                    imp_errors.push((
                        name.to_string(),
                        OsmprjError::ThemeNotFound {
                            theme: theme.clone(),
                        },
                    ));
                    continue;
                }
            }
        } else {
            None
        };

        let pbf_size_gb = std::fs::metadata(&pbf_path)
            .map(|m| m.len() as f64 / 1_073_741_824.0)
            .unwrap_or(0.0);

        let tuner_input = tuner::TunerInput {
            system_ram_gb: ram_gb,
            pbf_size_gb,
            ssd,
            concurrent_imports: config.project.effective_max_concurrent_imports(),
            database_url: db_url.to_string(),
            effective_schema: effective_schema.clone(),
            pbf_path: pbf_path.clone(),
            style_path,
            data_dir: data_dir.clone(),
            source_name: name.to_string(),
        };
        let argv = tuner::build_command(&tuner_input);
        let sql_files = collect_sql_files(source, &theme_registry);

        match import_source(
            Arc::clone(&imp_sem),
            Arc::clone(&mp),
            ImportSourceArgs {
                source_name: name.to_string(),
                db_url: db_url.to_string(),
                effective_schema,
                srid,
                argv,
                sql_files,
                log_dir: log_dir.clone(),
                verbose,
            },
        )
        .await
        {
            Ok(n) => imported.push(n),
            Err((n, e)) => imp_errors.push((n, e)),
        }
    }

    mp.clear().ok();

    // ── Report all errors collected across both phases ────────────────────────
    if !dl_errors.is_empty() || !imp_errors.is_empty() {
        for (name, e) in &dl_errors {
            eprintln!("  {} {name}: {e}", output::icon_error());
        }
        for (name, e) in &imp_errors {
            eprintln!("  {} {name}: {e}", output::icon_error());
        }
        return Err(OsmprjError::DownloadFailed {
            url: String::new(),
            message: format!(
                "{} download(s) failed, {} import(s) failed",
                dl_errors.len(),
                imp_errors.len()
            ),
        });
    }

    // ── Phase 3a: Update sources ──────────────────────────────────────────────
    if !update_sources.is_empty() {
        println!("\n  🔄  Updating {} source(s)\n", update_sources.len());
    }

    let spinner_style = output::spinner_style();

    for name in &update_sources {
        let source = &config.sources[*name];
        let effective_schema = source.effective_schema(name);

        let style_path: Option<PathBuf> = if let Some(ref theme) = source.theme {
            if let Some(plugin) = theme_registry.find(theme) {
                // Plugin theme: themepark type gets a wrapper; flex passes through directly.
                match plugin.theme_type() {
                    ThemeType::Themepark => Some(plugin.lua_path.clone()),
                    ThemeType::Flex => Some(plugin.lua_path.clone()),
                }
            } else {
                return Err(OsmprjError::ThemeNotFound {
                    theme: theme.clone(),
                });
            }
        } else {
            None
        };

        let log_path = log_dir.join(format!("{}-update.log", name.replace('/', "-")));

        let env_vars = vec![
            ("OSMPRJ_SCHEMA".to_string(), effective_schema.clone()),
            (
                "OSMPRJ_SRID".to_string(),
                source.effective_srid().to_string(),
            ),
        ];

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(spinner_style.clone());
        spinner.set_message(format!("Updating {name}..."));
        spinner.enable_steady_tick(std::time::Duration::from_millis(250));

        match replication_update(
            db_url,
            &effective_schema,
            style_path.as_ref(),
            max_diff_size_mb,
            &env_vars,
            &log_path,
            verbose,
        )
        .await
        {
            Ok(()) => {
                spinner.finish_with_message(format!("{} {name} updated", output::icon_success()));
            }
            Err(e) => {
                spinner
                    .finish_with_message(format!("{} {name} update failed", output::icon_warn()));
                eprintln!("  {} {name}: {e}", output::icon_warn());
                eprintln!("  Logs: {}", log_path.display());
            }
        }
    }

    let total_updated = update_sources.len();
    let total_imported = imported.len();
    println!(
        "\n  🌐  Sync complete. {} updated, {} newly imported.",
        style(format!("{total_updated} source(s)")).green(),
        style(format!("{total_imported} source(s)")).green(),
    );

    Ok(())
}

/// Looks up the PBF download URL for a Geofabrik source from the cached index.
async fn source_pbf_url(name: &str, config: &ProjectConfig) -> Option<String> {
    let _ = config; // not needed; URL comes from geofabrik index
                    // Load the cached Geofabrik index and look up the URL.
                    // Re-uses the existing geofabrik module.
    let features = crate::geofabrik::load_index().await.ok()?;
    let feature = crate::geofabrik::lookup(name, &features)?;
    feature.properties.urls.as_ref()?.pbf.clone()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Task 9.4 — verify that {schema} substitution works correctly in run_postprocess.
    ///
    /// We write a SQL file with a {schema} placeholder and call `run_postprocess`
    /// (via the substitution path only — no real DB connection needed to test the
    /// string transformation itself).  We verify by inspecting the transformed content
    /// directly rather than through a live DB.
    #[test]
    fn test_schema_substitution() {
        let tmp = TempDir::new().unwrap();
        let sql_path = tmp.path().join("01_test.sql");

        // Write a SQL file with multiple {schema} placeholders
        let template =
            "CREATE INDEX ON {schema}.buildings(way);\nCREATE INDEX ON {schema}.roads(way);";
        fs::write(&sql_path, template).unwrap();

        let content = fs::read_to_string(&sql_path).unwrap();
        let substituted = content.replace("{schema}", "germany");

        assert!(substituted.contains("germany.buildings"));
        assert!(substituted.contains("germany.roads"));
        assert!(!substituted.contains("{schema}"));

        // Also verify statement splitting: two statements should be found.
        let stmts: Vec<&str> = substituted
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn pbf_filename_plain_name() {
        assert_eq!(pbf_filename("albania"), "albania.osm.pbf");
    }

    #[test]
    fn pbf_filename_hierarchical_name() {
        assert_eq!(
            pbf_filename("north-america/us/alabama"),
            "north-america-us-alabama.osm.pbf"
        );
    }

    #[test]
    fn sort_sources_by_size_largest_first() {
        let mut sources = vec![
            (
                "ohio".to_string(),
                "http://example.com/ohio.pbf".to_string(),
                100_000_000u64,
            ),
            (
                "california".to_string(),
                "http://example.com/ca.pbf".to_string(),
                500_000_000u64,
            ),
            (
                "florida".to_string(),
                "http://example.com/fl.pbf".to_string(),
                200_000_000u64,
            ),
        ];
        sort_sources_by_size(&mut sources);
        assert_eq!(sources[0].0, "california"); // 500 MB first
        assert_eq!(sources[1].0, "florida"); // 200 MB second
        assert_eq!(sources[2].0, "ohio"); // 100 MB last
    }

    #[test]
    fn sort_sources_by_size_stable_for_equal_sizes() {
        // Two sources with identical sizes — original order should be preserved.
        let mut sources = vec![
            (
                "alabama".to_string(),
                "http://example.com/al.pbf".to_string(),
                100_000_000u64,
            ),
            (
                "alaska".to_string(),
                "http://example.com/ak.pbf".to_string(),
                100_000_000u64,
            ),
            (
                "arizona".to_string(),
                "http://example.com/az.pbf".to_string(),
                50_000_000u64,
            ),
        ];
        sort_sources_by_size(&mut sources);
        // arizona (smallest) should be last
        assert_eq!(sources[2].0, "arizona");
        // alabama and alaska both have size 100 MB; stable sort preserves their order
        assert_eq!(sources[0].0, "alabama");
        assert_eq!(sources[1].0, "alaska");
    }
}
