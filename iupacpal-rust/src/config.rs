use anyhow::{anyhow, Result};
use clap::CommandFactory;
use clap::Parser;
use seq_io::fasta::{Reader, Record};
use std::fs;

#[derive(Parser)]
pub struct Config {
    /// Input filename (FASTA).
    #[arg(short = 'f', default_value_t = String::from("input.fasta"))]
    pub input_file: String,

    /// Input sequence name.
    #[arg(short, default_value_t = String::from("seq0"))]
    pub seq_name: String,

    /// Minimum length.
    #[arg(short, default_value_t = 10)]
    pub min_len: i32,

    /// Maximum length.
    #[arg(short = 'M', default_value_t = 100)]
    pub max_len: i32,

    /// Maximum permissible gap.
    #[arg(short = 'g', default_value_t = 100)]
    pub max_gap: i32,

    /// Maximum permissible mismatches.
    #[arg(short = 'x', default_value_t = 0)]
    pub mismatches: i32,

    /// Output filename.
    #[arg(short, default_value_t = String::from("IUPACpalrs.out"))]
    pub output_file: String,

    /// Output format (classic, csv or custom_csv).
    #[arg(short = 'F', default_value_t = String::from("classic"))]
    pub output_format: String,
}

impl Config {
    #[allow(dead_code)] // ::new is actually used for testing...
    #[allow(clippy::too_many_arguments)]
    fn new(
        input_file: &str,
        seq_name: &str,
        min_len: i32,
        max_len: i32,
        max_gap: i32,
        mismatches: i32,
        output_file: &str,
        output_format: &str,
    ) -> Self {
        Self {
            input_file: input_file.to_string(),
            seq_name: seq_name.to_string(),
            min_len,
            max_len,
            max_gap,
            mismatches,
            output_file: output_file.to_string(),
            output_format: output_format.to_string(),
        }
    }

    pub fn from_args() -> Self {
        Config::parse()
    }

    #[allow(dead_code)] // For unit tests
    pub fn dummy(min_len: i32, max_len: i32, max_gap: i32, mismatches: i32) -> Self {
        Self {
            input_file: String::from("dummy"),
            seq_name: String::from("dummy"),
            min_len,
            max_len,
            max_gap,
            mismatches,
            output_file: String::from("dummy"),
            // initialize to classic to pass early Config::verify
            output_format: String::from("classic"),
        }
    }

    #[allow(dead_code)] // For unit tests
    pub fn dummy_default() -> Self {
        Config::dummy(10, 100, 100, 0)
    }

    // Just some clearer error handling (probably useless)
    fn check_file_exist(path: &str) -> Result<()> {
        let metadata = fs::metadata(path)
            .map_err(|_| anyhow!("'{}' does not exist or cannot access the path.", path))?;

        if metadata.is_file() {
            Ok(())
        } else {
            Err(anyhow!("'{}' is not a file", path))
        }
    }

    /// Attemps to extract the sequence (string) from the fasta file. Returns a trimmed lowercase String.
    ///
    /// If the sequence is not found, returns an Error with the list of found sequences.
    pub fn extract_string(&self) -> Result<String> {
        Config::check_file_exist(&self.input_file)?;
        let mut reader = Reader::from_path(&self.input_file)?;
        let mut found_seqs = Vec::new();
        while let Some(record) = reader.next() {
            let record = record.expect("Error reading record");
            let rec_id = record.id()?.to_owned();
            if rec_id == self.seq_name {
                return Ok(
                    std::str::from_utf8(record.seq())?
                        .to_lowercase()
                        .replace('\n', ""), // why isn't this the default?
                );
            } else {
                found_seqs.push(rec_id);
            }
        }

        Err(anyhow!(
            "Sequence {} not found. Found sequences in {} are:\n{}",
            &self.seq_name,
            &self.input_file,
            found_seqs.join("\n")
        ))
    }

    /// Attemps to extract the first sequence (string) from the fasta file. Returns a trimmed lowercase String.
    ///
    /// Returns an error if there are no sequences.
    ///
    /// Mainly used for convenience in test suites.
    #[allow(dead_code)] // For unit tests
    pub fn extract_first_string(input_file: String) -> Result<String> {
        Config::check_file_exist(&input_file)?;
        let mut reader = Reader::from_path(&input_file)?;
        let record = reader
            .next()
            .expect("No sequences found")
            .expect("Error reading record");

        Ok(std::str::from_utf8(record.seq())
            .unwrap()
            .to_lowercase()
            .replace('\n', ""))
    }

    pub fn verify(&self, n: usize) -> Result<()> {
        if let Err(msg) = Config::verify_bounds(self, n) {
            let _ = Config::command().print_help();
            println!();
            return Err(msg);
        }
        Config::verify_format(self)?;
        Ok(())
    }

    fn verify_bounds(&self, n: usize) -> Result<()> {
        if self.min_len as usize >= n {
            return Err(anyhow!(
                "min_len={} must be less than sequence length={}.",
                self.min_len,
                n
            ));
        }
        if self.max_gap as usize >= n {
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
        if self.mismatches as usize >= n {
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

    fn verify_format(&self) -> Result<()> {
        let allowed_formats = ["classic", "csv", "custom_csv"];
        if !allowed_formats.contains(&self.output_format.as_str()) {
            return Err(anyhow!(
                "Invalid output format. Allowed formats are: {}.",
                allowed_formats.join(", ")
            ));
        }
        Ok(())
    }

    // make this an instance of display?
    pub fn display(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("input_file:  {}\n", &self.input_file));
        out.push_str(&format!("seq_name:    {}\n", &self.seq_name));
        out.push_str(&format!("min_len:     {}\n", &self.min_len));
        out.push_str(&format!("max_len:     {}\n", &self.max_len));
        out.push_str(&format!("max_gap:     {}\n", &self.max_gap));
        out.push_str(&format!("mismatches:  {}\n", &self.mismatches));
        out.push_str(&format!("output_file: {}\n", &self.output_file));
        out.push_str(&format!("output_fmt:  {}\n", &self.output_format));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_output_format() {
        let config = Config::dummy_default();
        assert!(config.verify_format().is_ok())
    }

    #[test]
    fn test_invalid_output_format() {
        let config = Config::new("f", "f", 0, 0, 0, 0, "f", "wrong");
        assert!(config.verify_format().is_err())
    }
}
