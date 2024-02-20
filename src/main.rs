extern crate elapsed_time;

use iupacpal::config::Config;
use iupacpal::{find_palindromes, strinfigy_palindromes};

use anyhow::Result;
use std::fs::File;
use std::io::Write;

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    let config = Config::from_args();

    if config.seq_name != "ALL" {
        let seq = config.safe_extract_sequence()?;

        let palindromes = find_palindromes(&config, &seq);
        let out_str = strinfigy_palindromes(&config, &palindromes, &seq, 0)?;
    
        let mut file = File::create(&config.output_file)?;
        write!(&mut file, "{}", out_str)?;
    
        println!("\n{}", config.display());
        println!("Search complete!");
        println!("Found n={} palindromes", palindromes.len());
    } else {
        let records = config.safe_extract_all_records()?;

        let mut file = File::create(&config.output_file)?;
        
        for (idx, rec) in records.iter().enumerate() {
            let seq = &rec.sequence;
            let offset = rec.position;
            let n = seq.len();
            if let Err(e) = config.verify_bounds(n) {
                println!("Constraints violated for seq number {}", idx);
                println!("{}", e);
                continue;
            }
            let palindromes = find_palindromes(&config, &seq);
            let mut out_str = strinfigy_palindromes(&config, &palindromes, &seq, offset)?;
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
