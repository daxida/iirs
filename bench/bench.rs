extern crate anyhow;
extern crate iupacpal;

mod helper;
use helper::run_command;

use anyhow::Result;
use iupacpal::config;

use std::time::Instant;

const RUST_BINARY_PATH: &str = "target/release/iupacpal";
const RUST_OUTPUT_PATH: &str = "IUPACpalrs.out";

fn main() -> Result<()> {
    let start = Instant::now();

    // Modify these two
    let data = "alys";
    let times = 2;

    let config = match data {
        "alys" => config::Config {
            input_file: "tests/test_data/alys.fna".to_string(),
            seq_name: "NZ_CP059564.1".to_string(),
            params: config::SearchParams {
                min_len: 3,
                max_len: 100,
                max_gap: 20,
                mismatches: 0,
            },
            output_file: String::from(RUST_OUTPUT_PATH),
            output_format: "classic".to_string(),
        },
        "rand" => config::Config {
            input_file: "tests/test_data/rand10000000.fasta".to_string(),
            seq_name: "seq0".to_string(),
            params: config::SearchParams {
                min_len: 2,
                max_len: 100,
                max_gap: 5,
                mismatches: 1,
            },
            output_file: String::from(RUST_OUTPUT_PATH),
            output_format: "classic".to_string(),
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
