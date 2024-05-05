extern crate elapsed_time;

use iupacpal::config::Config;
use iupacpal::{find_palindromes, strinfigy_palindromes};

use anyhow::Result;
use std::fs::File;
use std::io::Write;

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    let config = Config::from_args();
    let seq = config.safe_extract_sequence()?;

    let palindromes = find_palindromes(&config.parameters, &seq)?;
    let out_str = strinfigy_palindromes(&config, &palindromes, &seq)?;

    let mut file = File::create(&config.output_file)?;
    writeln!(&mut file, "{}", out_str)?;

    println!("\n{}", config.display());
    println!("Search complete!");
    println!("Found n={} palindromes", palindromes.len());

    Ok(())
}
