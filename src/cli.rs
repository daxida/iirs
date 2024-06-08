use anyhow::Result;
use clap::Parser;

use crate::config::{Config, SearchParams};
use crate::constants::*;
use crate::utils::safe_extract_records;
use seq_io::fasta::{OwnedRecord, Record};

#[derive(Parser, Debug)]
pub struct Cli {
    /// Input filename (FASTA).
    #[arg(long, short = 'f', default_value_t = String::from(DEFAULT_INPUT_FILE))]
    pub input_file: String,

    /// Input sequence names (ids).
    #[arg(long, short, default_value = DEFAULT_SEQ_NAME, value_delimiter = ' ')]
    pub seq_names: Vec<String>,

    /// Minimum length.
    #[arg(long, short, default_value_t = DEFAULT_MIN_LEN)]
    pub min_len: usize,

    /// Maximum length.
    #[arg(long, short = 'M', default_value_t = DEFAULT_MAX_LEN)]
    pub max_len: usize,

    /// Maximum permissible gap.
    #[arg(long, short = 'g', default_value_t = DEFAULT_MAX_GAP)]
    pub max_gap: usize,

    /// Maximum permissible mismatches.
    #[arg(long, short = 'x', default_value_t = DEFAULT_MISMATCHES)]
    pub mismatches: usize,

    /// Output filename.
    /// For multiple sequences this is treated as a folder.
    #[arg(long, short, default_value_t = String::from(DEFAULT_OUTPUT_FILE))]
    pub output_file: String,

    /// Output format
    #[arg(long, short = 'F', default_value_t, value_enum)]
    pub output_format: OutputFormat,

    /// Quiet flag: Suppresses non-essential output when enabled.
    #[arg(long, short, default_value_t = false)]
    pub quiet: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Return a vector of pairs `(Config, OwnedRecord)` from the CLI arguments.
    ///
    /// The `check_bounds` argument determines if bound checking has to be performed for
    /// every sequence.
    ///
    /// The `Config` is different for every sequence since it contains the sequence name (id)
    /// and the output file. The SearchParams do not change.
    pub fn try_from_args(&self, check_bounds: bool) -> Result<Vec<(Config, OwnedRecord)>> {
        let params = SearchParams::new(self.min_len, self.max_len, self.max_gap, self.mismatches)?;
        let records = safe_extract_records(&self.input_file, &self.seq_names)?;
        let only_one_sequence_found = records.len() == 1;
        let mut config_record_pairs = Vec::new();

        for record in records {
            // I don't really like this leak hack to preserve the references
            // but the alternative of making everything a String is even worse.

            // IUPACpal convention is to always use IUPACpal.out no matter the sequence name.
            // In order to ease the validity checks, we keep that convention if the input consists
            // of only one sequence. Otherwise we preface the output_file with the sequence name.
            let seq_name = String::from(record.id()?);
            let this_output_file: Box<str> = if only_one_sequence_found {
                self.output_file.clone().into()
            } else {
                format!("{}/{}", self.output_file, seq_name).into_boxed_str()
            };

            let config = Config {
                input_file: &self.input_file,
                seq_name: Box::leak(seq_name.into_boxed_str()),
                params: params.clone(),
                output_file: Box::leak(this_output_file),
                output_format: self.output_format.clone(),
            };

            if check_bounds {
                config.params.check_bounds(record.seq.len())?;
            }
            config_record_pairs.push((config, record))
        }

        Ok(config_record_pairs)
    }
}
