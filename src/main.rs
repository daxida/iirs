extern crate elapsed_time;

use iirs::Cli;
use iirs::{find_irs, stringify_irs};

use anyhow::Result;
use std::fs::File;
use std::io::Write;

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    let args = Cli::parse_args();
    let check_bounds = false;
    let config_seq_pairs = args.try_from_args(check_bounds)?;
    let output_file = config_seq_pairs[0].0.output_file;
    let mut file = File::create(output_file)?;

    for (idx, (config, seq)) in config_seq_pairs.iter().enumerate() {
        if let Err(e) = config.params.check_bounds(seq.len()) {
            println!("Constraints violated for seq number {}", idx);
            println!("{}", e);
            continue;
        }

        // Extract position from the record id:
        // [location=10..86]
        let mut offset = 0;
        let keyword = "location=";
        if let Some(start_idx) = config.seq_name.find(keyword) {
            let start_idx = start_idx + keyword.len();

            if let Some(end_idx) = config.seq_name[start_idx..].find("..") {
                if let Ok(position_value) = config.seq_name[start_idx..start_idx + end_idx]
                    .trim()
                    .parse::<usize>()
                {
                    offset = position_value;
                }
            }
        }

        let irs = find_irs(&config.params, seq)?;
        let (header, irs_str) = stringify_irs(config, &irs, seq, offset);

        if idx == 0 {
            writeln!(&mut file, "{}", &header)?;
        }
        write!(&mut file, "{}", &irs_str)?;

        if !args.quiet {
            println!("\n{}", config);
            println!("Search complete for {}!", &config.seq_name);
            println!("Found n={} inverted repeats\n", irs.len());
        }
    }

    Ok(())
}
