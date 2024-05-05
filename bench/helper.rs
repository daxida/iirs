extern crate anyhow;
extern crate iupacpal;

use anyhow::{anyhow, Result};
use iupacpal::config;

use std::process::Command;
use std::time::{Duration, Instant};

pub fn run_command(cmd_beginning: &str, config: &config::Config) -> Result<Duration> {
    let start = Instant::now();

    let command = format!(
        "{} -f {} -s {} -m {} -M {} -g {} -x {}",
        cmd_beginning,
        config.input_file,
        config.seq_name,
        config.parameters.min_len,
        config.parameters.max_len,
        config.parameters.max_gap,
        config.parameters.mismatches
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("Error") {
        return Err(anyhow!("Error: (STDERR) {}", stderr));
    }

    if stdout.contains("Error") {
        return Err(anyhow!("Error: (STDOUT) {}", stdout));
    }

    Ok(start.elapsed())
}
