use iupacpal::config;

use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::{Duration, Instant};

fn run_command(cmd_beginning: &str, config: &config::Config) -> Result<Duration> {
    let start = Instant::now();

    let command = format!(
        "{} -f {} -s {} -m {} -M {} -g {} -x {}",
        cmd_beginning,
        config.input_file,
        config.seq_name,
        config.min_len,
        config.max_len,
        config.max_gap,
        config.mismatches
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("panic") {
        println!("Stderr: {}", stderr);
    }

    if stdout.contains("Error") {
        return Err(anyhow!("Error: {}", stdout));
    }

    Ok(start.elapsed())
}

fn main() -> Result<()> {
    let start = Instant::now();

    // Modify these two
    let data = "alys";
    let times = 10;

    let config = match data {
        "alys" => config::Config {
            input_file: "tests/test_data/alys.fna".to_string(),
            seq_name: "NZ_CP059564.1".to_string(),
            min_len: 3,
            max_len: 100,
            max_gap: 20,
            mismatches: 0,
            output_file: "IUPACpalrs.out".to_string(),
            output_format: "classic".to_string(),
        },
        "rand" => config::Config {
            input_file: "tests/test_data/rand10000000.fasta".to_string(),
            seq_name: "seq0".to_string(),
            min_len: 2,
            max_len: 100,
            max_gap: 5,
            mismatches: 1,
            output_file: "IUPACpalrs.out".to_string(),
            output_format: "classic".to_string(),
        },
        _ => todo!(),
    };

    for idx in 0..times {
        let dur = run_command("target/release/iupacpal", &config)?;
        println!("{}: took {}ms", idx, dur.as_millis());
    }

    println!(
        "\n-- Took {}ms on average.",
        start.elapsed().as_millis() / times
    );

    Ok(())
}
