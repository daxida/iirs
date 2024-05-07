use anyhow::{anyhow, Result};
use clap::Parser;
use seq_io::fasta::{Reader, Record};

use crate::utils;

const DEFAULT_MIN_LEN: usize = 10;
const DEFAULT_MAX_LEN: usize = 100;
const DEFAULT_MAX_GAP: usize = 100;
const DEFAULT_MISMATCHES: usize = 0;

const DEFAULT_INPUT_FILE: &str = "input.fasta";
const DEFAULT_SEQ_NAME: &str = "seq0";
const DEFAULT_OUTPUT_FILE: &str = "iirs.out";
const DEFAULT_OUTPUT_FORMAT: &str = "classic";

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

    /// Output format (classic, csv or custom_csv).
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

        Ok((config, seq))
    }
}

#[derive(Debug)]
pub struct SearchParams {
    pub min_len: usize,
    pub max_len: usize,
    pub max_gap: usize,
    pub mismatches: usize,
}

impl SearchParams {
    pub fn new(min_len: usize, max_len: usize, max_gap: usize, mismatches: usize) -> Result<Self> {
        if min_len < 2 {
            return Err(anyhow!("min_len={} must not be less than 2.", min_len));
        }
        if min_len > max_len {
            return Err(anyhow!(
                "min_len={} must be less than max_len={}.",
                min_len,
                max_len
            ));
        }
        if mismatches >= min_len {
            return Err(anyhow!(
                "mismatches={} must be less than min_len={}.",
                mismatches,
                min_len
            ));
        }

        Ok(Self {
            min_len,
            max_len,
            max_gap,
            mismatches,
        })
    }

    pub fn check_bounds(&self, n: usize) -> Result<()> {
        if self.min_len >= n {
            return Err(anyhow!(
                "min_len={} must be less than sequence length={}.",
                self.min_len,
                n
            ));
        }
        if self.max_gap >= n {
            return Err(anyhow!(
                "max_gap={} must be less than sequence length={}.",
                self.max_gap,
                n
            ));
        }

        if self.mismatches >= n {
            return Err(anyhow!(
                "mismatches={} must be less than sequence length={}.",
                self.mismatches,
                n
            ));
        }

        Ok(())
    }
}

impl Default for SearchParams {
    fn default() -> Self {
        SearchParams::new(
            DEFAULT_MIN_LEN,
            DEFAULT_MAX_LEN,
            DEFAULT_MAX_GAP,
            DEFAULT_MISMATCHES,
        )
        .unwrap()
    }
}

#[derive(Debug)]
pub struct Config<'a> {
    pub input_file: &'a str,
    pub seq_name: &'a str,
    pub params: SearchParams,
    pub output_file: &'a str,
    pub output_format: &'a str,
}

impl<'a> Config<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_file: &'a str,
        seq_name: &'a str,
        min_len: usize,
        max_len: usize,
        max_gap: usize,
        mismatches: usize,
        output_file: &'a str,
        output_format: &'a str,
    ) -> Self {
        Self {
            input_file,
            seq_name,
            params: SearchParams {
                min_len,
                max_len,
                max_gap,
                mismatches,
            },
            output_file,
            output_format,
        }
    }

    /// Attempts to extract the sequence with name 'seq_name' from the (fasta) input file.
    ///
    /// If the sequence is not found, returns an Error with the list of found sequences.
    pub fn safe_extract_sequence(input_file: &str, seq_name: &str) -> Result<Vec<u8>> {
        utils::check_file_exist(input_file)?;
        let mut reader = Reader::from_path(input_file)?;
        let mut found_seqs = Vec::new();
        while let Some(record) = reader.next() {
            let record = record.expect("Error reading record");
            let rec_id = record.id()?.to_owned();
            if rec_id == seq_name {
                let seq = utils::sanitize_sequence(record.seq())?;
                return Ok(seq);
            }

            found_seqs.push(rec_id);
        }

        Err(anyhow!(
            "Sequence '{}' not found.\nFound sequences in '{}' are:\n - {}",
            seq_name,
            input_file,
            found_seqs.join("\n - ")
        ))
    }
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Config {
            input_file: DEFAULT_INPUT_FILE,
            seq_name: DEFAULT_SEQ_NAME,
            params: SearchParams::default(),
            output_file: DEFAULT_OUTPUT_FILE,
            output_format: DEFAULT_OUTPUT_FORMAT,
        }
    }
}

impl<'a> std::fmt::Display for Config<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "input_file:  {}", self.input_file)?;
        writeln!(f, "seq_name:    {}", self.seq_name)?;
        writeln!(f, "min_len:     {}", self.params.min_len)?;
        writeln!(f, "max_len:     {}", self.params.max_len)?;
        writeln!(f, "max_gap:     {}", self.params.max_gap)?;
        writeln!(f, "mismatches:  {}", self.params.mismatches)?;
        writeln!(f, "output_file: {}", self.output_file)?;
        writeln!(f, "output_fmt:  {}", self.output_format)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_min_len_less_than_two() {
        assert!(SearchParams::new(0, 100, 0, 0).is_err());
    }
}
