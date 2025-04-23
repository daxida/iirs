extern crate elapsed_time;

use iirs::Cli;
use iirs::{find_irs, stringify_irs};

use anyhow::Result;
use std::fs::File;
use std::io::Write;

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    let args = Cli::parse_args();
    let check_bounds = true;
    let config_record_pairs = args.try_from_args(check_bounds)?;

    for (config, record) in config_record_pairs {
        let irs = find_irs(&config.params, &record.seq)?;
        let (header, irs_str) = stringify_irs(&config, &irs, &record.seq);

        let mut file = File::create(config.output_file)?;
        writeln!(&mut file, "{}\n{}", &header, &irs_str)?;

        if !args.quiet {
            println!("\n{config}");
            println!("Search complete for {}!", &config.seq_name);
            println!("Found n={} inverted repeats\n", irs.len());
        }
    }

    Ok(())
}
