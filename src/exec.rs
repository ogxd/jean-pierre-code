use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn run_cmd(cmd: &str, args: &[String]) -> Result<()> {
    let mut command = Command::new(cmd);
    command.args(args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn().with_context(|| format!("spawning '{}': {:?}", cmd, args))?;

    let output = child.wait_with_output()?;
    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;
    if !output.status.success() {
        anyhow::bail!("command '{}' failed with status {:?}", cmd, output.status);
    }
    Ok(())
}

pub fn cargo_build(args: &[String]) -> Result<()> {
    let mut all: Vec<String> = vec!["build".into()];
    all.extend_from_slice(args);
    run_cmd("cargo", &all)
}

pub fn cargo_test(args: &[String]) -> Result<()> {
    let mut all: Vec<String> = vec!["test".into()];
    all.extend_from_slice(args);
    run_cmd("cargo", &all)
}
