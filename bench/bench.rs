mod helper;
use helper::run_command;

use anyhow::Result;
use iirs::{Config, SearchParams};

use std::time::Instant;

const RUST_BINARY_PATH: &str = "target/release/iirs";
const RUST_OUTPUT_PATH: &str = "iirs.out";

fn main() -> Result<()> {
    let start = Instant::now();

    // Modify these two
    let data = "alys";
    let times = 2;

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
            output_format: iirs::OutputFormat::Classic,
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
            output_format: iirs::OutputFormat::Classic,
        },
        _ => todo!(),
    };

    for idx in 0..times {
        let dur = run_command(RUST_BINARY_PATH, &config)?;
        println!("{}: took {}ms", idx, dur.as_millis());
    }

    println!(
        "\nResults of bench against {}\n>> Took {}ms on average ({} runs).",
        data,
        start.elapsed().as_millis() / times,
        times
    );

    Ok(())
}
