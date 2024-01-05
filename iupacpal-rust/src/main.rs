extern crate elapsed_time;

use iupacpal::config::Config;
use iupacpal::find_palindromes;
use iupacpal::format::strinfigy_palindromes;

use anyhow::Result;
use std::fs::File;
use std::io::Write;

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    // Config and init variables
    let config = Config::from_args();
    let string = config.extract_string()?;
    let n = string.len();
    config.verify(n)?;
    let seq = string.as_bytes();

    // Find all palindromes
    let palindromes = find_palindromes(&config, &seq, n);

    // Stringify palindromes
    let out_str = strinfigy_palindromes(&config, &palindromes, &seq, n)?;

    // Write palindromes
    let mut file = File::create(&config.output_file)?;
    writeln!(&mut file, "{}", out_str)?;

    println!("\n{}", config.display());
    println!("Search complete!");
    println!("Found n={} palindromes", palindromes.len());

    Ok(())
}
