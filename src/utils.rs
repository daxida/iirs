use crate::constants::{EXISTING_FORMATS, IUPAC_SYMBOLS};
use anyhow::{anyhow, Result};
use std::fs;

/// Just some clearer error handling.
pub fn check_file_exist(path: &str) -> Result<()> {
    let metadata = fs::metadata(path)
        .map_err(|_| anyhow!("'{}' does not exist or cannot access the path.", path))?;

    if metadata.is_file() {
        Ok(())
    } else {
        Err(anyhow!("'{}' is not a file", path))
    }
}

/// Remove newlines, cast to lowercase and check that all the character are in IUPAC.
pub fn sanitize_sequence(seq: &[u8]) -> Result<Vec<u8>> {
    let mut sanitized_seq = Vec::new();

    for &byte in seq.iter() {
        if byte != b'\n' && byte != b'\r' {
            if !IUPAC_SYMBOLS.contains(byte.to_ascii_lowercase() as char) {
                return Err(anyhow!(
                    "sequence contains '{}' which is not an IUPAC symbol.",
                    byte as char
                ));
            }
            sanitized_seq.push(byte.to_ascii_lowercase());
        }
    }

    Ok(sanitized_seq)
}

/// Verify that the given format does actually exist.
pub fn verify_format(output_format: &str) -> Result<()> {
    if !EXISTING_FORMATS.contains(&output_format) {
        return Err(anyhow!(
            "Invalid output format. Allowed formats are: {}.",
            EXISTING_FORMATS.join(", ")
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_sequence_ok() {
        let seq = "acgturyswkmbdhvn*-".as_bytes().to_vec();
        assert!(sanitize_sequence(&seq).is_ok());
    }

    #[test]
    fn test_sanitize_sequence_newlines_one() {
        let seq = "acgturyswkmbdhvn*-\nacgturyswkmbdhvn*-".as_bytes().to_vec();
        let sanitized = sanitize_sequence(&seq).unwrap();
        let expected = "acgturyswkmbdhvn*-acgturyswkmbdhvn*-".as_bytes().to_vec();
        assert_eq!(expected, sanitized);
    }

    #[test]
    fn test_sanitize_sequence_newlines_two() {
        let seq = "acgturyswkmbdhvn*-\racgturyswkmbdhvn*-".as_bytes().to_vec();
        let sanitized = sanitize_sequence(&seq).unwrap();
        let expected = "acgturyswkmbdhvn*-acgturyswkmbdhvn*-".as_bytes().to_vec();
        assert_eq!(expected, sanitized);
    }

    #[test]
    fn test_sanitize_sequence_not_in_iupac() {
        let seq = "de".as_bytes().to_vec();
        assert!(sanitize_sequence(&seq).is_err());
    }

    #[test]
    fn test_valid_output_format() {
        assert!(verify_format("classic").is_ok());
        assert!(verify_format("csv").is_ok());
        assert!(verify_format("custom").is_ok())
    }

    #[test]
    fn test_invalid_output_format() {
        assert!(verify_format("wrong").is_err())
    }
}
