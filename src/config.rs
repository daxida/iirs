use anyhow::{anyhow, Result};
use seq_io::fasta::{Reader, Record};

use crate::constants::*;
use crate::utils;

#[derive(Debug, Clone)]
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
    ) -> Result<Self> {
        let params = SearchParams::new(min_len, max_len, max_gap, mismatches)?;
        Ok(Self {
            input_file,
            seq_name,
            params,
            output_file,
            output_format,
        })
    }

    /// Attempts to extract every sequence with name in `seq_names` from the input file.
    /// 
    /// If `seq_names` is only `ALL_SEQUENCES` then all the sequences are extracted.
    /// For example: `iirs -s ALL_SEQUENCES -m 5`
    ///
    /// Otherwise, if at least one sequence is not found, returns an Error with the list of missing 
    /// sequences, together with a list of all the sequences present in the input file.
    pub fn safe_extract_sequences(
        input_file: &str,
        seq_names: &[String],
    ) -> Result<Vec<(Vec<u8>, String)>> {
        // If `seq_names` is only `ALL_SEQUENCES` then all the sequences are extracted.
        let do_all_sequences = seq_names.len() == 1 && seq_names[0] == "ALL_SEQUENCES";

        utils::check_file_exist(input_file)?;

        let mut reader = Reader::from_path(input_file)?;
        let mut all_seqs_in_input_file = Vec::new();
        let mut seqs_found = Vec::new();
        let mut seqs_not_found_ids: Vec<String> = seq_names.to_vec();

        while let Some(record) = reader.next() {
            let record = record.expect("Error reading record");
            let rec_id = record.id()?.to_owned();
            if do_all_sequences || seq_names.contains(&rec_id) {
                let seq = utils::sanitize_sequence(record.seq())?;
                seqs_found.push((seq, rec_id.clone()));
                seqs_not_found_ids.retain(|id| id != &rec_id);
            }

            all_seqs_in_input_file.push(rec_id);
        }

        if !seqs_not_found_ids.is_empty() && !do_all_sequences {
            return Err(anyhow!(
                "Sequence(s) '{}' not found.\nFound sequences in '{}' are:\n - {}",
                seqs_not_found_ids.join(", "),
                input_file,
                all_seqs_in_input_file.join("\n - ")
            ));
        }

        Ok(seqs_found)
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
