use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub remote_endpoint: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub project_root: Option<String>,
}

fn config_paths() -> Result<(PathBuf, PathBuf)> {
    let proj = ProjectDirs::from("dev", "jean-pierre", "jean-pierre-code");
    let xdg = if let Some(p) = proj { p.config_dir().to_path_buf() } else { PathBuf::from(".jpc") };
    let local = PathBuf::from(".jpc");
    Ok((xdg, local))
}

pub fn init_config(force: bool) -> Result<()> {
    let (xdg_dir, local_dir) = config_paths()?;
    let cfg = default_config()?;
    let contents = toml::to_string_pretty(&cfg)?;

    for dir in [xdg_dir, local_dir] {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let path = dir.join("config.toml");
        if path.exists() && !force {
            continue;
        }
        fs::write(&path, &contents)?;
    }
    Ok(())
}

pub fn load_config() -> Result<Config> {
    let (xdg_dir, local_dir) = config_paths()?;
    let mut cfg = default_config()?;
    for path in [xdg_dir.join("config.toml"), local_dir.join("config.toml")] {
        if path.exists() {
            let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            let file_cfg: Config = toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
            cfg = merge(cfg, file_cfg);
        }
    }
    // Env overrides
    if let Ok(v) = std::env::var("JPC_REMOTE_ENDPOINT") { cfg.remote_endpoint = Some(v); }
    if let Ok(v) = std::env::var("JPC_API_KEY") { cfg.api_key = Some(v); }
    if let Ok(v) = std::env::var("JPC_MODEL") { cfg.model = Some(v); }
    if let Ok(v) = std::env::var("JPC_PROJECT_ROOT") { cfg.project_root = Some(v); }
    Ok(cfg)
}

fn merge(mut a: Config, b: Config) -> Config {
    if b.remote_endpoint.is_some() { a.remote_endpoint = b.remote_endpoint; }
    if b.api_key.is_some() { a.api_key = b.api_key; }
    if b.model.is_some() { a.model = b.model; }
    if b.project_root.is_some() { a.project_root = b.project_root; }
    a
}

fn default_config() -> Result<Config> {
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();
    Ok(Config {
        remote_endpoint: None,
        api_key: None,
        model: Some("tiny-llama".into()),
        project_root: Some(cwd),
    })
}

pub fn backups_dir() -> Result<PathBuf> {
    let d = Path::new(".jpc").join("backups");
    if !d.exists() { fs::create_dir_all(&d)?; }
    Ok(d)
}
