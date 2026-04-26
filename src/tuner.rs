use std::path::PathBuf;
use sysinfo::System;

pub struct TunerInput {
    pub system_ram_gb: f64,
    pub pbf_size_gb: f64,
    pub ssd: bool,
    pub database_url: String,
    pub effective_schema: String,
    pub pbf_path: PathBuf,
    pub style_path: PathBuf,
    pub data_dir: PathBuf,
    pub source_name: String,
}

pub fn system_ram_gb() -> f64 {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.total_memory() as f64 / 1_073_741_824.0 // bytes → GB
}

pub fn use_flat_nodes(pbf_size_gb: f64, ssd: bool) -> bool {
    (pbf_size_gb >= 8.0 && ssd) || pbf_size_gb >= 30.0
}

pub fn get_cache_mb(pbf_size_gb: f64, system_ram_gb: f64) -> u32 {
    let cache_max_gb = system_ram_gb * 0.66;
    let slim_cache_gb = 0.75 * (1.0 + 2.5 * pbf_size_gb);
    let chosen = slim_cache_gb.min(cache_max_gb);
    (chosen * 1024.0) as u32
}

pub fn build_command(input: &TunerInput) -> Vec<String> {
    let mut args = vec!["osm2pgsql".to_string()];

    args.push("--create".to_string());
    args.push("--slim".to_string());

    let flat = use_flat_nodes(input.pbf_size_gb, input.ssd);
    if flat {
        let nodes_path = input.data_dir.join(format!("{}.nodes", input.source_name));
        args.push(format!("--flat-nodes={}", nodes_path.display()));
        args.push("--cache=0".to_string());
    } else {
        let cache = get_cache_mb(input.pbf_size_gb, input.system_ram_gb);
        args.push(format!("--cache={cache}"));
    }

    args.push("--output=flex".to_string());
    args.push(format!("--style={}", input.style_path.display()));
    args.push(format!("--database={}", input.database_url));
    args.push(format!("--schema={}", input.effective_schema));
    args.push(input.pbf_path.display().to_string());

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn input(pbf_gb: f64, ram_gb: f64, ssd: bool) -> TunerInput {
        TunerInput {
            system_ram_gb: ram_gb,
            pbf_size_gb: pbf_gb,
            ssd,
            database_url: "postgres://localhost/osm".to_string(),
            effective_schema: "albania".to_string(),
            pbf_path: PathBuf::from("/data/albania.osm.pbf"),
            style_path: PathBuf::from("/tmp/style.lua"),
            data_dir: PathBuf::from("/data"),
            source_name: "albania".to_string(),
        }
    }

    #[test]
    fn small_file_no_flat_nodes() {
        let i = input(0.5, 16.0, true);
        assert!(!use_flat_nodes(i.pbf_size_gb, i.ssd));
        let cmd = build_command(&i);
        assert!(cmd.contains(&"--slim".to_string()));
        assert!(cmd.contains(&"--create".to_string()));
        assert!(!cmd.iter().any(|a| a.starts_with("--flat-nodes")));
        assert!(!cmd.iter().any(|a| a == "--drop"));
    }

    #[test]
    fn large_file_ssd_uses_flat_nodes() {
        let i = input(10.0, 16.0, true);
        assert!(use_flat_nodes(i.pbf_size_gb, i.ssd));
        let cmd = build_command(&i);
        assert!(cmd.iter().any(|a| a.starts_with("--flat-nodes=")));
        assert!(cmd.contains(&"--cache=0".to_string()));
    }

    #[test]
    fn very_large_file_non_ssd_uses_flat_nodes() {
        let i = input(35.0, 64.0, false);
        assert!(use_flat_nodes(i.pbf_size_gb, i.ssd));
        let cmd = build_command(&i);
        assert!(cmd.iter().any(|a| a.starts_with("--flat-nodes=")));
    }

    #[test]
    fn cache_capped_by_ram() {
        // 100 GB PBF, only 8 GB RAM → cache capped at 0.66 * 8 * 1024 = 5406 MB
        let cache = get_cache_mb(100.0, 8.0);
        let cap = (8.0 * 0.66 * 1024.0) as u32;
        let uncapped = (0.75 * (1.0 + 250.0) * 1024.0) as u32;
        assert!(cache <= cap);
        assert!(cache < uncapped);
    }

    #[test]
    fn command_always_has_slim_create_no_drop() {
        for (pbf, ram, ssd) in [(0.1, 4.0, true), (20.0, 32.0, true), (50.0, 128.0, false)] {
            let cmd = build_command(&input(pbf, ram, ssd));
            assert!(cmd.contains(&"--slim".to_string()), "missing --slim for pbf={pbf}");
            assert!(cmd.contains(&"--create".to_string()), "missing --create for pbf={pbf}");
            assert!(!cmd.iter().any(|a| a == "--drop"), "--drop present for pbf={pbf}");
        }
    }

    #[test]
    fn command_has_database_and_schema() {
        let cmd = build_command(&input(0.5, 16.0, true));
        assert!(cmd.iter().any(|a| a.starts_with("--database=")));
        assert!(cmd.iter().any(|a| a.starts_with("--schema=")));
    }
}
