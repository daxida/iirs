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

    /// Input sequence name.
    #[arg(short, default_value_t = String::from(DEFAULT_SEQ_NAME))]
    seq_name: String,

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
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    pub fn try_from_args(&self) -> Result<(Config, Vec<u8>)> {
        let params = SearchParams::new(self.min_len, self.max_len, self.max_gap, self.mismatches)?;

        let config = Config {
            input_file: &self.input_file,
            seq_name: &self.seq_name,
            params,
            output_file: &self.output_file,
            output_format: &self.output_format,
        };

        let seq = Config::safe_extract_sequence(&self.input_file, &self.seq_name)?;
        config.params.check_bounds(seq.len())?;
        utils::verify_format(&self.output_format)?;

        Ok((config, seq))
    }
}
