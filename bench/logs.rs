use anyhow::{anyhow, Result};
use clap::Parser;
use csv::WriterBuilder;
use itertools::iproduct;
use rand::prelude::SliceRandom;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};


const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

const SYMBOLS: [char; 17] = [
    'a', 'c', 'g', 't', 'u', 'r', 'y', 's', 'w', 'k', 'm', 'b', 'd', 'h', 'v', '*', '-',
];

fn generate_random_fasta(size_fasta: usize) -> String {
    let mut rng = rand::thread_rng();
    let random_sequence: String = (0..size_fasta)
        .map(|_| *SYMBOLS.choose(&mut rng).unwrap())
        .collect();

    format!(">seq0\n{}", random_sequence)
}

fn write_random_fasta(size_fasta: usize) -> Result<()> {
    let fasta = generate_random_fasta(size_fasta as usize);
    let mut file = File::create("rand.fasta").unwrap();
    file.write_all(fasta.as_bytes())?;

    Ok(())
}

fn run_command(cmd_beginning: &str, config: &BenchConfig) -> Result<f64> {
    let start = Instant::now();

    let command = format!(
        "{} -f {} -m {} -M {} -g {} -x {}",
        cmd_beginning,
        config.input_file,
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

    Ok(start.elapsed().as_secs_f64())
}

#[allow(dead_code)]
fn normalize_output(raw_output: &str) -> Vec<String> {
    raw_output
        .trim()
        .replace("Palindromes:\n", "")
        .split("\n\n")
        .skip(1)
        .map(|s| s.to_string())
        .collect()
}

#[allow(dead_code)]
fn test_equality() {
    let expected = fs::read_to_string("IUPACpal.out").unwrap();
    let received = fs::read_to_string("IUPACpalrs.out").unwrap();

    let expected_lines = normalize_output(&expected);
    let received_lines = normalize_output(&received);

    assert_eq!(expected_lines.len(), received_lines.len(),);

    for (el, rl) in expected_lines.iter().zip(received_lines.iter()) {
        assert_eq!(el, rl, "Received line:\n{}\nbut expected:\n{}", rl, el);
    }

    println!(
        "{}OK{}: Compared {} Palindromes",
        GREEN,
        RESET,
        expected_lines.len() - 1
    );
}

// fn average(timings: &[f64]) -> f64 {
//     let total: f64 = timings.iter().sum();
//     total / timings.len() as f64
// }

#[derive(Parser, Debug)]
struct BenchConfig {
    /// Input filename (FASTA).
    #[arg(short = 'f', default_value_t = String::from("rand.fasta"))]
    input_file: String,

    /// Input sequence name.
    #[arg(short, default_value_t = String::from("seq0"))]
    seq_name: String,

    /// Minimum length.
    #[arg(short, default_value_t = 10)]
    min_len: i32,

    /// Maximum length.
    #[arg(short = 'M', default_value_t = 100)]
    max_len: i32,

    /// Maximum permissible gap.
    #[arg(short = 'g', default_value_t = 100)]
    max_gap: i32,

    /// Maximum permissible mismatches.
    #[arg(short = 'x', default_value_t = 0)]
    mismatches: i32,

    /// Output filename.
    #[arg(short, default_value_t = String::from("IUPACpalrs.out"))]
    output_file: String,

    /// Output format (classic, csv or custom_csv).
    #[arg(short = 'F', default_value_t = String::from("classic"))]
    output_format: String,

    /// Size of the generated fasta
    #[arg(long, default_value_t = 1000)]
    size_fasta: usize,

    /// Number of tests to perform
    #[arg(long, default_value_t = 20)]
    n_tests: usize,
}

fn generate_configs<'a>(
    size_fasta: usize,
    steps: &'a Vec<Vec<i32>>,
) -> impl Iterator<Item = BenchConfig> + 'a {
    iproduct!(&steps[1], &steps[2], &steps[3]).map(move |(&min_len, &max_gap, &mismatches)| {
        BenchConfig {
            input_file: "rand.fasta".to_string(),
            seq_name: "seq0".to_string(),
            min_len,
            max_len: 100,
            max_gap,
            mismatches,
            output_file: "IUPACpalrs.out".to_string(),
            output_format: "classic".to_string(),
            size_fasta,
            n_tests: 20,
        }
    })
}

fn main() -> Result<()> {
    let start = Instant::now();

    let steps: Vec<Vec<i32>> = vec![
        // size_fasta
        vec![10000],
        // min_len
        vec![2, 4, 6, 8, 10, 12, 14, 16],
        // max_gap
        vec![0, 1, 2, 3, 4, 5],
        // mismatches
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
    ];

    let mut writer = WriterBuilder::new().from_writer(File::create("bench/results.csv")?);

    writer.write_record([
        "size_fasta",
        "min_len",
        "max_gap",
        "mismatches",
        "cpp_timing",
        "rust_timing",
    ])?;

    let parallel = false;

    if parallel {
        let writer = Arc::new(Mutex::new(writer));

        for &size_fasta in &steps[0] {
            write_random_fasta(size_fasta as usize)?;
    
            generate_configs(size_fasta as usize, &steps)
                .collect::<Vec<_>>()
                .into_par_iter()
                .for_each(|config| {
                    let ctiming = run_command("IUPACpal/IUPACpal", &config);
                    let rtiming = run_command("target/release/iupacpal", &config);
            
                    if let (Ok(ctiming), Ok(rtiming)) = (ctiming, rtiming) {
                        let mut writer = writer.lock().unwrap();
    
                        writer.write_record(&[
                            size_fasta.to_string(),
                            config.min_len.to_string(),
                            config.max_gap.to_string(),
                            config.mismatches.to_string(),
                            ctiming.to_string(),
                            rtiming.to_string(),
                        ]).unwrap();
                    }
                });
        }
    } else {
        for &size_fasta in &steps[0] {
            write_random_fasta(size_fasta as usize)?;
    
            for config in generate_configs(size_fasta as usize, &steps) {
                let ctiming = run_command("IUPACpal/IUPACpal", &config);
                let rtiming = run_command("target/release/iupacpal", &config);
        
                if let (Ok(ctiming), Ok(rtiming)) = (ctiming, rtiming) {
                    writer.write_record(&[
                        size_fasta.to_string(),
                        config.min_len.to_string(),
                        config.max_gap.to_string(),
                        config.mismatches.to_string(),
                        ctiming.to_string(),
                        rtiming.to_string(),
                    ])?;
    
                    test_equality();
                }
            }
        }
    }

    println!(
        "\n{}OK{}: All tests finished in {}",
        GREEN,
        RESET,
        start.elapsed().as_secs_f64()
    );

    Ok(())
}
