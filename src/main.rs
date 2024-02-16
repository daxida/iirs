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

    if config.seq_name != "ALL" {
        let seq = config.safe_extract_sequence()?;

        // Find all palindromes
        let palindromes = find_palindromes(&config, &seq);
    
        // Stringify palindromes
        let out_str = strinfigy_palindromes(&config, &palindromes, &seq)?;
    
        // Write palindromes
        let mut file = File::create(&config.output_file)?;
        write!(&mut file, "{}", out_str)?;
    
        println!("\n{}", config.display());
        println!("Search complete!");
        println!("Found n={} palindromes", palindromes.len());
    } else {
        let seqs = config.safe_extract_all_sequences()?;

        let mut file = File::create(&config.output_file)?;
        
        for (idx, seq) in seqs.iter().enumerate() {
            let n = seq.len();
            if let Err(e) = config.verify_bounds(n) {
                println!("Constraints violated for seq number {}", idx);
                println!("{}", e);
                continue;
            }
            let palindromes = find_palindromes(&config, seq);
            let mut out_str = strinfigy_palindromes(&config, &palindromes, seq)?;
            let mut lines = out_str.lines();

            // Skip headers (hacky)
            if idx > 0 {
                lines.next(); 
            }
            
            out_str = lines.collect::<Vec<_>>().join("\n");

            writeln!(&mut file, "{}", out_str)?;
        }
    
        println!("\n{}", config.display());
        println!("Search complete!");
    }

    Ok(())
}
