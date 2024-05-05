use anyhow::{anyhow, Result};
use clap::CommandFactory;
use clap::Parser;
use seq_io::fasta::{Reader, Record};

use crate::utils;

#[derive(Parser, Debug)]
pub struct SearchParams {
    /// Minimum length.
    #[arg(short, default_value_t = 10)]
    pub min_len: usize,

    /// Maximum length.
    #[arg(short = 'M', default_value_t = 100)]
    pub max_len: usize,

    /// Maximum permissible gap.
    #[arg(short = 'g', default_value_t = 100)]
    pub max_gap: usize,

    /// Maximum permissible mismatches.
    #[arg(short = 'x', default_value_t = 0)]
    pub mismatches: usize,
}

#[derive(Parser, Debug)]
pub struct Config {
    /// Input filename (FASTA).
    #[arg(short = 'f', default_value_t = String::from("input.fasta"))]
    pub input_file: String,

    /// Input sequence name.
    #[arg(short, default_value_t = String::from("seq0"))]
    pub seq_name: String,

    #[clap(flatten)]
    pub params: SearchParams,

    /// Output filename.
    #[arg(short, default_value_t = String::from("IUPACpalrs.out"))]
    pub output_file: String,

    /// Output format (classic, csv or custom_csv).
    #[arg(short = 'F', default_value_t = String::from("classic"))]
    pub output_format: String,
}

impl SearchParams {
    pub fn new(min_len: usize, max_len: usize, max_gap: usize, mismatches: usize) -> Self {
        Self {
            min_len,
            max_len,
            max_gap,
            mismatches,
        }
    }

    /// Verify that the given search parameters make sense (f.e. min_len < max_len).
    pub fn verify_bounds(&self, n: usize) -> Result<()> {
        if self.min_len < 2 {
            return Err(anyhow!("min_len={} must not be less than 2.", self.min_len));
        }
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
        if self.min_len > self.max_len {
            return Err(anyhow!(
                "min_len={} must be less than max_len={}.",
                self.min_len,
                self.max_len
            ));
        }
        if self.mismatches >= n {
            return Err(anyhow!(
                "mismatches={} must be less than sequence length={}.",
                self.mismatches,
                n
            ));
        }
        if self.mismatches >= self.min_len {
            return Err(anyhow!(
                "mismatches={} must be less than min_len={}.",
                self.mismatches,
                self.min_len
            ));
        }

        Ok(())
    }
}

impl Default for SearchParams {
    fn default() -> Self {
        // Same as the clap defaults.
        SearchParams::new(10, 100, 100, 0)
    }
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_file: &str,
        seq_name: &str,
        min_len: usize,
        max_len: usize,
        max_gap: usize,
        mismatches: usize,
        output_file: &str,
        output_format: &str,
    ) -> Self {
        Self {
            input_file: input_file.to_string(),
            seq_name: seq_name.to_string(),
            params: SearchParams {
                min_len,
                max_len,
                max_gap,
                mismatches,
            },
            output_file: output_file.to_string(),
            output_format: output_format.to_string(),
        }
    }

    pub fn from_args() -> Self {
        Config::parse()
    }

    /// Attempts to extract the sequence with name 'seq_name' from the (fasta) input file.
    ///
    /// If the sequence is not found, returns an Error with the list of found sequences.
    pub fn safe_extract_sequence(&self) -> Result<Vec<u8>> {
        utils::check_file_exist(&self.input_file)?;
        let mut reader = Reader::from_path(&self.input_file)?;
        let mut found_seqs = Vec::new();
        while let Some(record) = reader.next() {
            let record = record.expect("Error reading record");
            let rec_id = record.id()?.to_owned();
            if rec_id == self.seq_name {
                let seq = utils::sanitize_sequence(record.seq())?;
                Config::verify(self, seq.len())?;
                return Ok(seq);
            }

            found_seqs.push(rec_id);
        }

        Err(anyhow!(
            "Sequence '{}' not found.\nFound sequences in '{}' are:\n - {}",
            &self.seq_name,
            &self.input_file,
            found_seqs.join("\n - ")
        ))
    }

    pub fn verify(&self, n: usize) -> Result<()> {
        if let Err(msg) = SearchParams::verify_bounds(&self.params, n) {
            let _ = Config::command().print_help();
            println!();
            return Err(msg);
        }
        utils::verify_format(&self.output_format)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            input_file: String::new(),
            seq_name: String::new(),
            params: SearchParams::default(),
            output_file: String::new(),
            // To not crash on stringify palindromes
            output_format: String::from("classic"),
        }
    }
}

impl std::fmt::Display for Config {
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
        let params = SearchParams::new(0, 100, 0, 0);
        assert!(params.verify_bounds(10).is_err());
    }
}
