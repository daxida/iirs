use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::time::Instant;

mod helper;
use helper::run_command;

use anyhow::Result;
use iirs::{Config, SearchParams};

const RUST_BINARY_PATH: &str = "target/release/iirs";
const RUST_OUTPUT_PATH: &str = "iirs.out";
// Change to None to print only to stdout
const OUTPUT_FILE: Option<&str> = Some("bench/bench_result.txt");

fn main() -> Result<()> {
    let start = Instant::now();

    let data = "alys";
    let times = 10;
    let output_format = iirs::OutputFormat::Classic;

    let config = match data {
        "alys" => Config {
            input_file: "tests/test_data/alys.fna",
            seq_name: "NZ_CP059564.1",
            params: SearchParams {
                min_len: 3,
                max_len: 100,
                max_gap: 20,
                mismatches: 0,
            },
            output_file: RUST_OUTPUT_PATH,
            output_format,
        },
        "rand" => Config {
            input_file: "tests/test_data/rand1000000.fasta",
            seq_name: "seq0",
            params: SearchParams {
                min_len: 2,
                max_len: 100,
                max_gap: 5,
                mismatches: 1,
            },
            output_file: RUST_OUTPUT_PATH,
            output_format,
        },
        _ => todo!(),
    };

    let mut output: Box<dyn Write> = match OUTPUT_FILE {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(stdout()),
    };

    let config_str = format!("{config}\n");
    if OUTPUT_FILE.is_some() {
        print!("{config_str}");
    }
    output.write_all(config_str.as_bytes())?;

    for idx in 0..times {
        let dur = run_command(RUST_BINARY_PATH, &config)?;
        let line = format!("{}: took {}ms\n", idx, dur.as_millis());
        if OUTPUT_FILE.is_some() {
            print!("{}", line);
        }
        output.write_all(line.as_bytes())?;
    }

    let summary = format!(
        "\nResults of bench against {}\n>> Took {}ms on average ({} runs).\n",
        data,
        start.elapsed().as_millis() / times,
        times
    );
    if OUTPUT_FILE.is_some() {
        print!("{}", summary);
    }
    output.write_all(summary.as_bytes())?;

    Ok(())
}
