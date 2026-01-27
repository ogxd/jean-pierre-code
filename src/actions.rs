use crate::config;
use crate::exec;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub description: String,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    WriteFile { path: String, content: String, #[serde(default)] create_dirs: bool },
    Run { cmd: String, args: Vec<String> },
}

impl Action {
    pub fn short(&self) -> String {
        match self {
            Action::WriteFile { path, .. } => format!("write_file:{}", path),
            Action::Run { cmd, args } => format!("run:{} {}", cmd, args.join(" ")),
        }
    }
}

pub fn apply_plan(plan: &Plan) -> Result<()> {
    for act in &plan.actions {
        match act {
            Action::WriteFile { path, content, create_dirs } => {
                let p = PathBuf::from(path);
                if *create_dirs {
                    if let Some(parent) = p.parent() { fs::create_dir_all(parent)?; }
                }
                if p.exists() {
                    let backups = config::backups_dir()?;
                    let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
                    let fname = p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("file");
                    let bpath = backups.join(format!("{}-{}.bak", fname, ts));
                    let data = fs::read(&p).with_context(|| format!("reading {}", p.display()))?;
                    fs::write(&bpath, data)?;
                }
                fs::write(&p, content).with_context(|| format!("writing {}", p.display()))?;
            }
            Action::Run { cmd, args } => {
                exec::run_cmd(cmd, args)?;
            }
        }
    }
    Ok(())
}
