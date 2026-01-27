mod cli;
mod config;
mod context;
mod local_llm;
mod remote;
mod actions;
mod exec;

use anyhow::{Context as _, Result};
use cli::{Cli, Commands};
use clap::Parser;
use log::{debug, info};

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => {
            config::init_config(force)?;
            println!("Initialized configuration for jean-pierre-code.");
        }
        Commands::Context { max_files, max_bytes } => {
            let ctx = context::gather_context(max_files, max_bytes)?;
            println!("{}", serde_json::to_string_pretty(&ctx)?);
        }
        Commands::Plan { query, max_tokens } => {
            let cfg = config::load_config()?;
            let ctx = context::gather_context(None, None)?;
            let planner = local_llm::build_local_llm(&cfg)?;
            let plan = planner.plan_actions(&ctx, &query, max_tokens.unwrap_or(2048))?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Commands::Apply { plan_file, dry_run } => {
            let text = std::fs::read_to_string(&plan_file)
                .with_context(|| format!("reading plan file: {}", plan_file))?;
            let actions: actions::Plan = serde_json::from_str(&text)
                .with_context(|| "parsing plan JSON")?;
            if dry_run {
                println!("Would apply {} actions:", actions.actions.len());
                for (i, a) in actions.actions.iter().enumerate() {
                    println!("{:03}: {}", i + 1, a.short());
                }
            } else {
                actions::apply_plan(&actions)?;
                println!("Applied {} actions.", actions.actions.len());
            }
        }
        Commands::Chat { prompt } => {
            let cfg = config::load_config()?;
            let remote = remote::build_remote(&cfg)?;
            let ctx = context::gather_context(Some(10), Some(256_000))?;
            let content = format!(
                "User: {}\n\nContext (truncated): {}",
                prompt,
                context::truncate_for_prompt(&ctx, 2000)
            );
            let response = remote.generate(&content, 2048)?;
            println!("{}", response);
        }
        Commands::Run { what } => {
            match what.as_str() {
                "build" => exec::cargo_build(&[])?,
                "test" => exec::cargo_test(&[])?,
                other => exec::run_cmd(other, &[])?,
            }
        }
    }

    Ok(())
}
