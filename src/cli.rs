use anyhow::Result;
use clap::Parser;

use crate::config::{Config, SearchParams};
use crate::constants::*;
use crate::utils;

#[derive(Parser, Debug)]
pub struct Cli {
    /// Input filename (FASTA).
    #[arg(short = 'f', default_value_t = String::from(DEFAULT_INPUT_FILE))]
    input_file: String,

    /// Input sequence names.
    #[arg(short, default_value = DEFAULT_SEQ_NAME, value_delimiter = ' ')]
    seq_names: Vec<String>,

    /// Minimum length.
    #[arg(short, default_value_t = DEFAULT_MIN_LEN)]
    min_len: usize,

    /// Maximum length.
    #[arg(short = 'M', default_value_t = DEFAULT_MAX_LEN)]
    max_len: usize,

    /// Maximum permissible gap.
    #[arg(short = 'g', default_value_t = DEFAULT_MAX_GAP)]
    max_gap: usize,

    /// Maximum permissible mismatches.
    #[arg(short = 'x', default_value_t = DEFAULT_MISMATCHES)]
    mismatches: usize,

    /// Output filename.
    #[arg(short, default_value_t = String::from(DEFAULT_OUTPUT_FILE))]
    output_file: String,

    /// Output format (classic, csv or custom).
    #[arg(short = 'F', default_value_t = String::from(DEFAULT_OUTPUT_FORMAT))]
    output_format: String,

    /// Quiet flag: Suppresses non-essential output when enabled.
    #[arg(short, default_value_t = false)]
    pub quiet: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Return a vector of pairs (Config, sequence) from the CLI arguments.
    pub fn try_from_args(&self) -> Result<Vec<(Config, Vec<u8>)>> {
        utils::verify_format(&self.output_format)?;

        let params = SearchParams::new(self.min_len, self.max_len, self.max_gap, self.mismatches)?;
        let seqs_with_names = Config::safe_extract_sequences(&self.input_file, &self.seq_names)?;
        let only_one_sequence_found = seqs_with_names.len() == 1;
        let mut results = Vec::new();

        for (seq, seq_name) in seqs_with_names {
            // I don't really like this leak hack to preserve the references
            // but the alternative of making everything a String is even worse.

            // IUPACpal convention is to always use IUPACpal.out no matter the sequence name.
            // In order to ease the validity checks, we keep that convention if the input consists
            // of only one sequence. Otherwise we preface the output_file with the sequence name.
            let this_output_file: Box<str> = if only_one_sequence_found {
                self.output_file.clone().into()
            } else {
                format!("{}_{}", seq_name, self.output_file).into_boxed_str()
            };

            let config = Config {
                input_file: &self.input_file,
                seq_name: Box::leak(seq_name.into_boxed_str()),
                params: params.clone(),
                output_file: Box::leak(this_output_file),
                output_format: &self.output_format,
            };

            config.params.check_bounds(seq.len())?;
            results.push((config, seq))
        }

        Ok(results)
    }
}
