use anyhow::{anyhow, Result};
use clap::Parser;
use seq_io::fasta::{Reader, Record};
use std::fs;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input filename (FASTA).
    #[arg(short = 'f', long, default_value_t = String::from("input.fasta"))]
    input_file: String,

    /// Input sequence name.
    #[arg(short, long, default_value_t = String::from("seq0"))]
    seq_name: String,

    /// Minimum length.
    #[arg(short, long, default_value_t = 10)]
    min_len: i32,

    /// Maximum length.
    #[arg(short = 'M', long, default_value_t = 100)]
    max_len: i32,

    /// Maximum permissible gap.
    #[arg(short = 'g', long, default_value_t = 100)]
    max_gap: i32,

    /// Maximum permissible mismatches.
    #[arg(short = 'x', long, default_value_t = 0)]
    mismatches: i32,

    /// Output filename.
    #[arg(short, long, default_value_t = String::from("IUPACpalrs.out"))]
    output_file: String,
}

#[derive(Debug)]
pub struct Config {
    pub input_file: String,
    pub seq_name: String,
    pub min_len: i32,
    pub max_len: i32,
    pub max_gap: i32,
    pub mismatches: i32,
    pub output_file: String,
}

impl Config {
    pub fn new(
        input_file: &str,
        seq_name: &str,
        min_len: i32,
        max_len: i32,
        max_gap: i32,
        mismatches: i32,
        output_file: &str,
    ) -> Self {
        Self {
            input_file: input_file.to_string(),
            seq_name: seq_name.to_string(),
            min_len,
            max_len,
            max_gap,
            mismatches,
            output_file: output_file.to_string(),
        }
    }

    // TODO: gets replaced by clap
    // HOWTO: print the clap message error instead of this usage_string
    fn usage() {
        println!("\n  FLAG  PARAMETER       TYPE      DEFAULT         DESCRIPTION");
        println!("  -f    input_file      <str>     input.fasta     Input filename (FASTA).");
        println!("  -s    seq_name        <str>     seq0            Input sequence name.");
        println!("  -m    min_len         <int>     10              Minimum length.");
        println!("  -M    max_len         <int>     100             Maximum length.");
        println!("  -g    max_gap         <int>     100             Maximum permissible gap.");
        println!(
            "  -x    mismatches      <int>     0               Maximum permissible mismatches."
        );
        println!("  -o    output_file     <str>     IUPACpal.out    Output filename.");
        println!();
    }

    pub fn from_args() -> Self {
        let args = Args::parse();
        Config::new(
            args.input_file.as_str(),
            args.seq_name.as_str(),
            args.min_len,
            args.max_len,
            args.max_gap,
            args.mismatches,
            args.output_file.as_str(),
        )
    }

    /// Attemps to extract the sequence from the fasta file.
    /// Returns a trimmed String in lowercase.
    /// If the sequence is not found, returns an Error with the sequences that were actually
    /// present in the fasta for convenience.
    pub fn extract_string(&self) -> Result<String> {
        let mut reader = Reader::from_path(self.input_file.as_str()).unwrap();
        let mut found_seqs = Vec::new();
        while let Some(record) = reader.next() {
            let record = record.expect("Error reading record");
            let rec_id = record.id().unwrap().to_owned();
            if rec_id == self.seq_name {
                return Ok(std::str::from_utf8(&record.seq())?
                    .trim_end()
                    .to_lowercase()
                    .replace('\n', "") // why isn't this the default?
                );
            } else {
                found_seqs.push(rec_id);
            }
        }

        let err_msg = format!(
            "Sequence {} not found. Found sequences in {} are:\n{}",
            &self.seq_name,
            &self.input_file,
            found_seqs.join("\n")
        );
        Err(anyhow!(err_msg))
    }

    // TODO: finish
    pub fn verify(&self, n: usize) -> Result<()> {
        if let Ok(metadata) = fs::metadata(&self.input_file) {
            if !metadata.is_file() {
                return Err(anyhow!("File '{}' not found", &self.input_file));
            }
        }

        // Verify arguments are valid with respect to individual limits
        if self.max_gap < 0 {
            return Err(anyhow!("max_gap must not be a negative value."));
        }

        // Verify arguments are valid with respect to each other
        if self.max_gap as usize >= n {
            Config::usage();
            return Err(anyhow!("max_gap={} must be less than sequence length={}.", self.max_gap, n));
        }
        if self.max_len < self.min_len {
            Config::usage();
            return Err(anyhow!("max_len must not be less than min_len."));   
        }
        if self.mismatches as usize >= n {
            Config::usage();
            return Err(anyhow!("mismatches must be less than sequence length."));      
        }
        if self.mismatches >= self.min_len {
            Config::usage();
            return Err(anyhow!("mismatches must be less than min_len."));      
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
        out.push_str("\n");
        out
    }

    pub fn out_palindrome_display(&self, n: usize) -> String {
        let config_out = format!(
            "Palindromes of: {}\n\
            Sequence name: {}\n\
            Sequence length is: {}\n\
            Start at position: {}\n\
            End at position: {}\n\
            Minimum length of Palindromes is: {}\n\
            Maximum length of Palindromes is: {}\n\
            Maximum gap between elements is: {}\n\
            Number of mismatches allowed in Palindrome: {}\n\n\n\n\
            Palindromes:\n",
            &self.input_file,
            &self.seq_name,
            n,
            1,
            n,
            self.min_len,
            self.max_len,
            self.max_gap,
            self.mismatches,
        );

        config_out
    }
}
