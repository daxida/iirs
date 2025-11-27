use anyhow::{bail, Result};
use iirs::Config;

use std::process::Command;
use std::time::{Duration, Instant};

pub fn run_command(cmd_beginning: &str, config: &Config) -> Result<Duration> {
    let start = Instant::now();

    let command = format!(
        "{} -f {} -s {} -m {} -M {} -g {} -x {}",
        cmd_beginning,
        config.input_file,
        config.seq_name,
        config.params.min_len,
        config.params.max_len,
        config.params.max_gap,
        config.params.mismatches
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("Error") {
        eprintln!("Ran command: {command}");
        bail!("(STDERR) {}", stderr)
    }

    if stdout.contains("Error") {
        eprintln!("Ran command: {command}");
        bail!("(STDOUT) {}", stdout)
    }

    Ok(start.elapsed())
}
