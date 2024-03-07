extern crate anyhow;
extern crate csv;
extern crate itertools;
extern crate iupacpal;
extern crate rand;
extern crate rayon;

use anyhow::{anyhow, Result};
use csv::WriterBuilder;
use itertools::iproduct;
use iupacpal::config::Config;
use rand::prelude::SliceRandom;
use rayon::prelude::*;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

const SYMBOLS: [char; 17] = [
    'a', 'c', 'g', 't', 'u', 'r', 'y', 's', 'w', 'k', 'm', 'b', 'd', 'h', 'v', '*', '-',
];
// const SYMBOLS: [char; 5] = ['a', 'c', 'g', 't', 'n'];

fn generate_random_fasta(size_fasta: usize) -> String {
    let mut rng = rand::thread_rng();
    let random_sequence: String = (0..size_fasta)
        .map(|_| *SYMBOLS.choose(&mut rng).unwrap())
        .collect();

    format!(">seq0\n{}", random_sequence)
}

fn write_random_fasta(size_fasta: usize) -> Result<()> {
    let fasta = generate_random_fasta(size_fasta);
    let mut file = File::create("rand.fasta").unwrap();
    file.write_all(fasta.as_bytes())?;

    Ok(())
}

fn run_command(cmd_beginning: &str, config: &Config) -> Result<f64> {
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

    if stderr.contains("Error") {
        return Err(anyhow!("Error: {}", stderr));
    }
    if stdout.contains("Error") {
        return Err(anyhow!("Error: {}", stdout));
    }

    Ok(start.elapsed().as_secs_f64())
}

#[allow(dead_code)]
fn normalize_output(raw_output: &str) -> Vec<&str> {
    raw_output.trim().lines().collect()
}

#[allow(dead_code)]
fn test_equality() -> Result<()> {
    let expected = fs::read_to_string("IUPACpal.out").unwrap();
    let received = fs::read_to_string("IUPACpalrs.out").unwrap();

    let expected_lines = normalize_output(&expected);
    let received_lines = normalize_output(&received);

    let expected_size = expected_lines.len();
    let received_size = received_lines.len();

    if expected_size != received_size {
        // Known bug in the cpp implementation where it doesn't detect the only palindrome.
        if expected_size == 13 && received_size == 16 {
            return Ok(());
        }

        return Err(anyhow!(
            "Different lengths:\ncpp has {} lines\nrst has {} lines",
            expected_size,
            received_size
        ));
    }

    for (el, rl) in expected_lines.iter().zip(received_lines.iter()) {
        if el != rl {
            return Err(anyhow!("Received line:\n{}\nbut expected:\n{}", rl, el));
        }
    }

    // println!(
    //     "{}OK{}: Compared {} Palindromes",
    //     GREEN,
    //     RESET,
    //     expected_lines.len() - 1
    // );

    Ok(())
}

// fn average(timings: &[f64]) -> f64 {
//     let total: f64 = timings.iter().sum();
//     total / timings.len() as f64
// }

fn generate_configs(steps: &[Vec<usize>]) -> impl Iterator<Item = Config> + '_ {
    iproduct!(&steps[1], &steps[2], &steps[3]).map(move |(&min_len, &max_gap, &mismatches)| {
        Config {
            input_file: "rand.fasta".to_string(),
            seq_name: "seq0".to_string(),
            min_len,
            max_len: 100,
            max_gap,
            mismatches,
            output_file: "DUMMY".to_string(),
            output_format: "classic".to_string(),
        }
    })
}

fn main() -> Result<()> {
    let start = Instant::now();

    // let steps: Vec<Vec<i32>> = vec![
    //     // size_fasta
    //     vec![10000],
    //     // min_len
    //     vec![2, 4, 6, 8, 10, 12, 14, 16],
    //     // max_gap
    //     vec![0, 1, 2, 3, 4, 5],
    //     // mismatches
    //     vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
    // ];
    let n_tests = 1;
    let steps: Vec<Vec<usize>> = vec![
        // size_fasta
        vec![1000, 10000, 100000],
        // min_len
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20],
        // max_gap
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20],
        // mismatches
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20],
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

            generate_configs(&steps)
                .collect::<Vec<_>>()
                .into_par_iter()
                .for_each(|config| {
                    let ctiming = run_command("IUPACpal/IUPACpal", &config);
                    let rtiming = run_command("target/release/iupacpal", &config);

                    if let (Ok(ctiming), Ok(rtiming)) = (ctiming, rtiming) {
                        let mut writer = writer.lock().unwrap();

                        writer
                            .write_record(&[
                                size_fasta.to_string(),
                                config.min_len.to_string(),
                                config.max_gap.to_string(),
                                config.mismatches.to_string(),
                                ctiming.to_string(),
                                rtiming.to_string(),
                            ])
                            .unwrap();
                    }
                });
        }
    } else {
        for &size_fasta in &steps[0] {
            for config in generate_configs(&steps) {
                // The config doesn't make sense: skip
                if config.verify_bounds(size_fasta).is_err() {
                    continue;
                }
                for _ in 0..n_tests {
                    write_random_fasta(size_fasta as usize)?;
                    let ctiming = run_command("IUPACpal/IUPACpal", &config);
                    let rtiming = run_command("target/release/iupacpal", &config);

                    match (ctiming, rtiming) {
                        (Ok(ctiming), Ok(rtiming)) => {
                            writer.write_record(&[
                                size_fasta.to_string(),
                                config.min_len.to_string(),
                                config.max_gap.to_string(),
                                config.mismatches.to_string(),
                                ctiming.to_string(),
                                rtiming.to_string(),
                            ])?;

                            if let Err(msg) = test_equality() {
                                println!("{:?}", &config);
                                return Err(msg);
                            }
                        }
                        (Err(c_err), Ok(_)) => {
                            println!("Cpp failed but rs succeeded: {:?}", c_err);
                            return Err(c_err);
                        }
                        (Ok(_), Err(r_err)) => {
                            println!("Rs failed but cpp succeeded: {:?}", r_err);
                            return Err(r_err);
                        }
                        (Err(_), Err(_)) => {
                            // Both commands failed (wrong inputs)
                            panic!()
                        }
                    }
                }
            }

            println!("OK: tests with size {}", size_fasta);
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
