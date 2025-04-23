use crate::constants::IUPAC_SYMBOLS;
use anyhow::{Result, anyhow};
use seq_io::fasta::{OwnedRecord, Reader, Record};
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

    for &byte in seq {
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

/// Attempts to extract the record of every sequence with id in `seq_ids` from the input file.
///
/// If `seq_ids` is only `ALL_SEQUENCES` then all the sequences are extracted.
/// For example:
///
/// `iirs -s ALL_SEQUENCES -m 5`
///
/// If at least one sequence is not found, returns an error with the list of missing
/// sequences, together with a list of all the sequences present in the input file.
pub fn safe_extract_records(input_file: &str, seq_ids: &[String]) -> Result<Vec<OwnedRecord>> {
    check_file_exist(input_file)?;

    let do_all_sequences = seq_ids.len() == 1 && seq_ids[0] == "ALL_SEQUENCES";

    let mut reader = Reader::from_path(input_file)?;
    let mut all_seq_ids_found = Vec::new();
    let mut seq_ids_not_found = seq_ids.to_vec();
    let mut records = Vec::new();

    while let Some(record) = reader.next() {
        let record = record.expect("Error reading record");
        let mut owned_record = record.to_owned_record();
        let record_id = owned_record.id()?.to_string();
        if do_all_sequences || seq_ids.contains(&record_id) {
            owned_record.seq = sanitize_sequence(record.seq())?;
            records.push(owned_record);
            seq_ids_not_found.retain(|id| id != &record_id);
        }

        all_seq_ids_found.push(record_id);
    }

    if !seq_ids_not_found.is_empty() && !do_all_sequences {
        return Err(anyhow!(
            "Sequence(s) '{}' not found.\nFound sequences in '{}' are:\n - {}",
            seq_ids_not_found.join(", "),
            input_file,
            all_seq_ids_found.join("\n - ")
        ));
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_sequence_ok() {
        let seq = b"acgturyswkmbdhvn*-".to_vec();
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
        let seq = b"de".to_vec();
        assert!(sanitize_sequence(&seq).is_err());
    }
}
