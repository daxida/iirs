mod helper;
use helper::run_command;

use anyhow::{anyhow, Result};
use clap::Parser;
use csv::WriterBuilder;
use iirs::config::{Config, SearchParams};
use itertools::iproduct;
use rand::prelude::SliceRandom;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

const RANDOM_FILE_PATH: &str = "rand.fasta";

const CPP_BINARY_PATH: &str = "bench/IUPACpal";
const RUST_BINARY_PATH: &str = "target/release/iirs";
const CPP_OUTPUT_PATH: &str = "IUPACpal.out";
const RUST_OUTPUT_PATH: &str = "iirs.out"; // This is the default anyway

const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

const SYMBOLS: [char; 17] = [
    'a', 'c', 'g', 't', 'u', 'r', 'y', 's', 'w', 'k', 'm', 'b', 'd', 'h', 'v', '*', '-',
];
// const SYMBOLS: [char; 5] = ['a', 'c', 'g', 't', 'n'];

struct TestSuite {
    n_test: usize,
    size_seq: Vec<usize>,
    min_len: Vec<usize>,
    max_gap: Vec<usize>,
    mismatches: Vec<usize>,
}

impl TestSuite {
    fn manual() -> Self {
        TestSuite {
            n_test: 1,
            size_seq: vec![1000],
            min_len: vec![2, 4, 6, 8, 10, 12, 14, 16],
            max_gap: vec![0, 1, 2, 3, 4, 5],
            mismatches: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    fn random(size_seq: usize, n_test: usize) -> Self {
        TestSuite {
            n_test,
            size_seq: vec![size_seq],
            min_len: vec![10],
            max_gap: vec![100],
            mismatches: vec![2],
        }
    }

    // Return a cartesian product of Configs.
    fn to_configs_iter(&self) -> impl Iterator<Item = Config> + '_ {
        let TestSuite {
            min_len,
            max_gap,
            mismatches,
            ..
        } = self;

        iproduct!(
            min_len.iter().cloned(),
            max_gap.iter().cloned(),
            mismatches.iter().cloned()
        )
        .map(move |(min_len, max_gap, mismatches)| Config {
            input_file: RANDOM_FILE_PATH,
            params: SearchParams {
                min_len,
                max_len: 100,
                max_gap,
                mismatches,
            },
            ..Default::default()
        })
    }
}

#[derive(Parser, Debug)]
pub struct Runner {
    /// Print more information about timings.
    #[arg(long, default_value_t = false)]
    pub verbose: bool,

    /// Whether to write the results in a csv or not.
    #[arg(long, default_value_t = false)]
    pub write: bool,

    /// Start a random bench.
    /// The first arg is the size of the sequence, then the number of tests.
    #[clap(long, num_args = 2)]
    pub random_bench: Vec<usize>,
}

impl Runner {
    fn get_test_suite(&self) -> TestSuite {
        match self.random_bench.as_slice() {
            [size_seq, n_test] => TestSuite::random(*size_seq, *n_test),
            _ => TestSuite::manual(),
        }
    }
}

fn generate_random_fasta(size_seq: usize) -> String {
    let mut rng = rand::thread_rng();
    let random_sequence: String = (0..size_seq)
        .map(|_| *SYMBOLS.choose(&mut rng).unwrap())
        .collect();

    format!(">seq0\n{}", random_sequence)
}

fn write_random_fasta(size_seq: usize) -> Result<()> {
    let fasta = generate_random_fasta(size_seq);
    let mut file = File::create(RANDOM_FILE_PATH).unwrap();
    file.write_all(fasta.as_bytes())?;

    Ok(())
}

fn normalize_output(raw_output: &str) -> Vec<&str> {
    raw_output.trim().lines().collect()
}

fn test_equality(runner: &Runner) -> Result<()> {
    let expected = fs::read_to_string(CPP_OUTPUT_PATH).unwrap();
    let received = fs::read_to_string(RUST_OUTPUT_PATH).unwrap();

    let expected_lines = normalize_output(&expected);
    let received_lines = normalize_output(&received);

    let expected_size = expected_lines.len();
    let received_size = received_lines.len();

    if expected_size != received_size {
        // Known bug in the cpp implementation where it doesn't detect the only IR.
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

    if runner.verbose {
        println!(
            "{}OK{}: Compared {} Palindromes",
            GREEN,
            RESET,
            expected.split("\n\n").collect::<Vec<_>>().len() - 1
        );
    }

    Ok(())
}

fn average(timings: &[Duration]) -> f64 {
    let total_millis: u128 = timings.iter().map(|&d| d.as_millis()).sum();
    let total_seconds = total_millis as f64 / 1000.0;
    total_seconds / timings.len() as f64
}

fn main() -> Result<()> {
    let start = Instant::now();

    let runner = Runner::parse();
    let test_suite = runner.get_test_suite();

    let mut writer = if runner.write {
        Some(WriterBuilder::new().from_writer(File::create("bench/results.csv")?))
    } else {
        None
    };
    if let Some(ref mut writer) = writer {
        writer.write_record([
            "size_seq",
            "min_len",
            "max_gap",
            "mismatches",
            "cpp_timing",
            "rust_timing",
        ])?;
    }

    for size_seq in &test_suite.size_seq {
        let mut ctimings: Vec<Duration> = Vec::new();
        let mut rtimings: Vec<Duration> = Vec::new();

        for config in test_suite.to_configs_iter() {
            // The config doesn't make sense: skip
            if let Err(_) = config.params.check_bounds(*size_seq) {
                // println!("{}", &err);
                continue;
            }
            for _ in 0..test_suite.n_test {
                write_random_fasta(*size_seq)?;
                let ctiming = run_command(CPP_BINARY_PATH, &config);
                let rtiming = run_command(RUST_BINARY_PATH, &config);

                match (ctiming, rtiming) {
                    (Ok(ctiming), Ok(rtiming)) => {
                        if let Some(ref mut writer) = writer {
                            writer.write_record(&[
                                size_seq.to_string(),
                                config.params.min_len.to_string(),
                                config.params.max_gap.to_string(),
                                config.params.mismatches.to_string(),
                                ctiming.as_secs_f64().to_string(),
                                rtiming.as_secs_f64().to_string(),
                            ])?;
                        }

                        ctimings.push(ctiming);
                        rtimings.push(rtiming);

                        if let Err(msg) = test_equality(&runner) {
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

        if runner.verbose {
            println!(
                "Results for {} random tests of size {}.",
                test_suite.n_test, size_seq
            );
            println!("cpp  average: {:.4}", average(&ctimings));
            println!("rust average: {:.4}", average(&rtimings));
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
