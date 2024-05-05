use anyhow::{anyhow, Result};
use clap::CommandFactory;
use clap::Parser;
use seq_io::fasta::{Reader, Record};

use crate::utils;

#[derive(Parser, Debug)]
pub struct Parameters {
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
    pub parameters: Parameters,

    /// Output filename.
    #[arg(short, default_value_t = String::from("IUPACpalrs.out"))]
    pub output_file: String,

    /// Output format (classic, csv or custom_csv).
    #[arg(short = 'F', default_value_t = String::from("classic"))]
    pub output_format: String,
}

impl Parameters {
    pub fn new(min_len: usize, max_len: usize, max_gap: usize, mismatches: usize) -> Self {
        Self {
            min_len,
            max_len,
            max_gap,
            mismatches,
        }
    }

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

impl Config {
    #[allow(dead_code)] // ::new is actually used for testing...
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
            parameters: Parameters {
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

    #[allow(dead_code)] // For unit tests
    pub fn dummy(min_len: usize, max_len: usize, max_gap: usize, mismatches: usize) -> Self {
        Self {
            input_file: String::from("dummy"),
            seq_name: String::from("dummy"),
            parameters: Parameters {
                min_len,
                max_len,
                max_gap,
                mismatches,
            },
            output_file: String::from("dummy"),
            // initialize to classic to pass early Config::verify
            output_format: String::from("classic"),
        }
    }

    #[allow(dead_code)] // For unit tests
    pub fn dummy_default() -> Self {
        Config::dummy(10, 100, 100, 0)
    }

    /// Attemps to extract the first sequence (string) from the fasta file. Returns a trimmed lowercase String.
    ///
    /// Returns an error if there are no sequences.
    ///
    /// Mainly used for convenience in unit tests.
    #[allow(dead_code)]
    pub fn extract_first_string(path: &str) -> Result<String> {
        utils::check_file_exist(path)?;
        let mut reader = Reader::from_path(path)?;
        let record = reader
            .next()
            .expect("No sequences found")
            .expect("Error reading record");

        Ok(std::str::from_utf8(record.seq())
            .unwrap()
            .to_lowercase()
            .replace('\n', ""))
    }

    /// Attempts to extract the sequence from the (fasta) input file.
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
            "Sequence {} not found. Found sequences in {} are:\n{}",
            &self.seq_name,
            &self.input_file,
            found_seqs.join("\n")
        ))
    }

    pub fn verify(&self, n: usize) -> Result<()> {
        if let Err(msg) = Parameters::verify_bounds(&self.parameters, n) {
            let _ = Config::command().print_help();
            println!();
            return Err(msg);
        }
        utils::verify_format(&self.output_format)?;
        Ok(())
    }

    // make this an instance of display?
    pub fn display(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("input_file:  {}\n", &self.input_file));
        out.push_str(&format!("seq_name:    {}\n", &self.seq_name));
        out.push_str(&format!("min_len:     {}\n", &self.parameters.min_len));
        out.push_str(&format!("max_len:     {}\n", &self.parameters.max_len));
        out.push_str(&format!("max_gap:     {}\n", &self.parameters.max_gap));
        out.push_str(&format!("mismatches:  {}\n", &self.parameters.mismatches));
        out.push_str(&format!("output_file: {}\n", &self.output_file));
        out.push_str(&format!("output_fmt:  {}\n", &self.output_format));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_min_len_less_than_two() {
        let config = Parameters::new(0, 100, 0, 0);
        assert!(config.verify_bounds(10).is_err());
    }
}
