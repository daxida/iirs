extern crate elapsed_time;

use iupacpal::config::Config;
use iupacpal::{find_irs, stringify_irs};

use anyhow::Result;
use std::fs::File;
use std::io::Write;

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    let config = Config::from_args();
    let seq = config.safe_extract_sequence()?;

    let irs = find_irs(&config.params, &seq)?;
    let out_str = stringify_irs(&config, &irs, &seq)?;

    let mut file = File::create(&config.output_file)?;
    writeln!(&mut file, "{}", out_str)?;

    println!("\n{}", config);
    println!("Search complete!");
    println!("Found n={} inverted repeats", irs.len());

    Ok(())
}
