use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::constants::{
    DEFAULT_INPUT_FILE, DEFAULT_MAX_GAP, DEFAULT_MAX_LEN, DEFAULT_MIN_LEN, DEFAULT_MISMATCHES,
    DEFAULT_OUTPUT_FILE, DEFAULT_SEQ_NAME, OutputFormat,
};

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
            bail!("min_len={min_len} must not be less than 2.")
        }
        if min_len > max_len {
            bail!("min_len={min_len} must be less than max_len={max_len}.")
        }
        if mismatches >= min_len {
            bail!("mismatches={mismatches} must be less than min_len={min_len}.",)
        }

        Ok(Self {
            min_len,
            max_len,
            max_gap,
            mismatches,
        })
    }

    // Note that if max_gap >= n, the result is the same as if it was equal to n.
    pub fn check_bounds(&self, n: usize) -> Result<()> {
        if self.min_len >= n {
            bail!(
                "min_len={} must be less than sequence length={}.",
                self.min_len,
                n
            )
        }
        if self.mismatches >= n {
            bail!(
                "mismatches={} must be less than sequence length={}.",
                self.mismatches,
                n
            )
        }

        Ok(())
    }
}

impl Default for SearchParams {
    fn default() -> Self {
        Self::new(
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
    pub output_path: PathBuf,
    pub output_format: OutputFormat,
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
        output_path: &'a str,
        output_format: OutputFormat,
    ) -> Result<Self> {
        let params = SearchParams::new(min_len, max_len, max_gap, mismatches)?;
        Ok(Self {
            input_file,
            seq_name,
            params,
            output_path: output_path.into(),
            output_format,
        })
    }
}

impl Default for Config<'_> {
    fn default() -> Self {
        Config {
            input_file: DEFAULT_INPUT_FILE,
            seq_name: DEFAULT_SEQ_NAME,
            params: SearchParams::default(),
            output_path: PathBuf::from(DEFAULT_OUTPUT_FILE),
            output_format: OutputFormat::default(),
        }
    }
}

impl std::fmt::Display for Config<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "input_file:  {}", self.input_file)?;
        writeln!(f, "seq_name:    {}", self.seq_name)?;
        writeln!(f, "min_len:     {}", self.params.min_len)?;
        writeln!(f, "max_len:     {}", self.params.max_len)?;
        writeln!(f, "max_gap:     {}", self.params.max_gap)?;
        writeln!(f, "mismatches:  {}", self.params.mismatches)?;
        writeln!(f, "output_path: {}", self.output_path.display())?;
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
