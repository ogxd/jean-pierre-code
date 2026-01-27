use anyhow::{Context as _, Result};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ContextSnapshot {
    pub cwd: String,
    pub git: Option<GitInfo>,
    pub files: Vec<FileSnippet>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitInfo {
    pub branch: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileSnippet {
    pub path: String,
    pub bytes: usize,
    pub content: String,
}

pub fn gather_context(max_files: Option<usize>, max_bytes: Option<usize>) -> Result<ContextSnapshot> {
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();
    let git = git_info();

    let mut files: Vec<PathBuf> = Vec::new();
    // Heuristic include list
    for p in ["Cargo.toml", "Cargo.lock"] {
        let pb = PathBuf::from(p);
        if pb.exists() { files.push(pb); }
    }
    collect_dir(&mut files, Path::new("src"))?;
    collect_dir(&mut files, Path::new("tests")).ok();

    let max_files = max_files.unwrap_or(50);
    let max_bytes = max_bytes.unwrap_or(512_000);
    let mut used_bytes = 0usize;
    let mut out: Vec<FileSnippet> = Vec::new();
    for path in files.into_iter().take(max_files) {
        if !path.exists() { continue; }
        let data = fs::read_to_string(&path).unwrap_or_default();
        let mut content = data;
        // Trim long files
        if content.len() > 64_000 { content.truncate(64_000); }
        let bytes = content.as_bytes().len();
        if used_bytes + bytes > max_bytes { break; }
        used_bytes += bytes;
        out.push(FileSnippet { path: path.to_string_lossy().to_string(), bytes, content });
    }

    Ok(ContextSnapshot { cwd, git, files: out })
}

pub fn truncate_for_prompt(ctx: &ContextSnapshot, max_chars: usize) -> String {
    // A very light text rendering of the context
    let mut s = String::new();
    s.push_str(&format!("cwd: {}\n", ctx.cwd));
    if let Some(g) = &ctx.git {
        s.push_str(&format!("git: branch={:?}\n", g.branch));
    }
    for f in ctx.files.iter() {
        s.push_str(&format!("--- {} ({} bytes) ---\n", f.path, f.bytes));
        s.push_str(&f.content);
        s.push('\n');
        if s.len() > max_chars { break; }
    }
    if s.len() > max_chars { s.truncate(max_chars); }
    s
}

fn collect_dir(out: &mut Vec<PathBuf>, dir: &Path) -> Result<()> {
    if !dir.exists() { return Ok(()); }
    for entry in walkdir::WalkDir::new(dir).max_depth(4) {
        let e = entry?;
        if e.file_type().is_file() {
            let p = e.into_path();
            out.push(p);
        }
    }
    Ok(())
}

fn git_info() -> Option<GitInfo> {
    let branch = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output().ok().and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None });
    let status = Command::new("git").args(["status", "--porcelain"]).output().ok().and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).to_string()) } else { None });
    if branch.is_none() && status.is_none() { return None; }
    Some(GitInfo { branch, status })
}
