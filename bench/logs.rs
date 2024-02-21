use anyhow::{anyhow, Result};
use clap::Parser;
use csv::WriterBuilder;
use rand::prelude::SliceRandom;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

fn generate_random_fasta(size: usize) -> String {
    let mut rng = rand::thread_rng();
    format!(
        ">seq0\n{}",
        (0..size)
            .map(|_| {
                *[
                    "a", "c", "g", "t", "u", "r", "y", "s", "w", "k", "m", "b", "d", "h", "v", "*",
                    "-",
                ]
                .choose(&mut rng)
                .unwrap()
            })
            .collect::<String>()
    )
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
        // println!("Error: {}", stdout);
        // std::process::exit(0);
    }

    Ok(start.elapsed().as_secs_f64())
}

fn normalize_output(raw_output: &str) -> Vec<String> {
    let output = raw_output.trim().replace("Palindromes:\n", "");
    let lines: Vec<String> = output
        .split("\n\n")
        .skip(1)
        .map(|s| s.to_string())
        .collect();

    // Sort then to compare when trying alternative sorting strategies in find_palindromes

    // fn block_sort(block: &str) -> (usize, usize, usize, usize) {
    //     if block.trim().is_empty() {
    //         return (1_000_000_000, 0, 0, 0);
    //     }
    //     let lines: Vec<&str> = block.split('\n').collect();
    //     let mut iter1 = lines[0].split_whitespace();

    //     let a = iter1.next().unwrap();
    //     let _ = iter1.next();
    //     let c = iter1.next().unwrap();

    //     let mut iter2 = lines[2].split_whitespace();
    //     let d = iter2.next().unwrap();
    //     let _ = iter2.next();
    //     let f = iter2.next().unwrap();

    //     (
    //         a.parse().unwrap(),
    //         c.parse().unwrap(),
    //         d.parse().unwrap(),
    //         f.parse().unwrap(),
    //     )
    // }

    // let mut lines = lines.clone();
    // lines.sort_by(|a, b| block_sort(a).cmp(&block_sort(b)));

    lines
}

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

fn main() {
    let start = Instant::now();

    let steps: Vec<Vec<i32>> = vec![
        // size_fasta
        vec![100, 1000, 10000, 100000],
        // min_len
        vec![2, 3, 4, 5, 10],
        // max_gap
        vec![0, 5, 10, 20, 50],
        // mismatches
        vec![0, 1, 2, 3, 4, 5, 10],
    ];

    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_writer(File::create("bench/results.csv").expect("Failed to create CSV file"));

    writer.write_record(&["size_fasta", "min_len", "max_gap", "mismatches", "cpp_timing", "rust_timing"]).expect("Failed to write CSV header");


    for &size_fasta in &steps[0] {
        let fasta = generate_random_fasta(size_fasta as usize);
        let mut file = File::create("rand.fasta").unwrap();
        file.write_all(fasta.as_bytes())
            .expect("Failed to write to file");

        for &min_len in &steps[1] {
            for &max_gap in &steps[2] {
                for &mismatches in &steps[3] {
                    let config = BenchConfig {
                        input_file: "rand.fasta".to_string(),
                        seq_name: "seq0".to_string(),
                        min_len,
                        max_len: 100,
                        max_gap,
                        mismatches,
                        output_file: "IUPACpalrs.out".to_string(),
                        output_format: "classic".to_string(),
                        size_fasta: size_fasta as usize,
                        n_tests: 20,
                    };
                    // println!("{:?}", config);

                    // TODO: compute average over 20 tries?

                    let cpp_timing = run_command("IUPACpal/IUPACpal", &config);
                    let rust_timing = run_command("target/release/main", &config);

                    match (cpp_timing, rust_timing) {
                        (Ok(cpp_timing), Ok(rust_timing)) => {
                            writer
                                .write_record(&[
                                    size_fasta.to_string(),
                                    min_len.to_string(),
                                    max_gap.to_string(),
                                    mismatches.to_string(),
                                    cpp_timing.to_string(),
                                    rust_timing.to_string(),
                                ])
                                .expect("Failed to write CSV record");

                            test_equality();
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    println!("\nAll tests finished in {}", start.elapsed().as_secs_f64());
}
