extern crate elapsed_time;

use iirs::find_irs;
use iirs::{Cli, OutputFormat};

use anyhow::Result;
use std::fs::File;
use std::io::Write;

fn fmt_custom_header() -> String {
    String::from("ir_start,motif,gap_motif,reverse_complement")
}

fn fmt_custom_with_offset(
    irs: &Vec<(usize, usize, usize)>,
    seq: &[u8],
    offset: usize,
) -> String {
    let mut out_str = String::new();

    for (left, right, gap) in irs {
        let outer_left = left + 1;
        let outer_right = right + 1;
        let inner_left = (outer_left + outer_right - 1 - gap) / 2;
        let inner_right = (outer_right + outer_left + 1 + gap) / 2;

        let nucleotide = (*left..inner_left)
            .map(|i| seq[i] as char)
            .collect::<String>();
        let gap_nucleotide = (inner_left..(inner_right - 1))
            .map(|i| seq[i] as char)
            .collect::<String>();
        let reverse_complement = ((inner_right - 1)..outer_right)
            .rev()
            .map(|i| seq[i] as char)
            .collect::<String>();

        out_str.push_str(&format!(
            "{},{},{},{}\n",
            outer_left + offset,
            nucleotide,
            gap_nucleotide,
            reverse_complement
        ));
    }

    out_str
}

fn extract_offset_from_record_head(seq_name: &str) -> usize {
    // Extract position from the record head:
    // [location=10..86]
    let mut offset = 0;

    let keyword = "location=";
    if let Some(start_idx) = seq_name.find(keyword) {
        let start_idx = start_idx + keyword.len();

        if let Some(end_idx) = seq_name[start_idx..].find("..") {
            if let Ok(position_value) = seq_name[start_idx..start_idx + end_idx]
                .trim()
                .parse::<usize>()
            {
                offset = position_value;
            }
        }
    }

    offset
}

#[elapsed_time::elapsed]
fn main() -> Result<()> {
    let args = Cli::parse_args();
    assert_eq!(args.output_format, OutputFormat::Custom);

    let check_bounds = false;
    let config_seq_pairs = args.try_from_args(check_bounds)?;
    let output_file = &args.output_file;
    let mut file = File::create(output_file)?;

    write!(&mut file, "{}", &fmt_custom_header())?;

    for (idx, (config, seq)) in config_seq_pairs.iter().enumerate() {
        if let Err(e) = config.params.check_bounds(seq.len()) {
            println!("Constraints violated for seq number {}", idx);
            println!("{}", e);
            continue;
        }

        let offset = extract_offset_from_record_head(config.seq_name);

        let irs = find_irs(&config.params, seq)?;
        let irs_str = fmt_custom_with_offset(&irs, seq, offset);
        write!(&mut file, "{}", &irs_str)?;

        if !args.quiet {
            println!("\n{}", config);
            println!("Search complete for {}!", &config.seq_name);
            println!("Found n={} inverted repeats\n", irs.len());
        }
    }

    Ok(())
}
