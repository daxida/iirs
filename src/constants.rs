use std::collections::BTreeSet;

pub const DEFAULT_MIN_LEN: usize = 10;
pub const DEFAULT_MAX_LEN: usize = 100;
pub const DEFAULT_MAX_GAP: usize = 100;
pub const DEFAULT_MISMATCHES: usize = 0;

pub const DEFAULT_INPUT_FILE: &str = "input.fasta";
pub const DEFAULT_SEQ_NAME: &str = "seq0";
pub const DEFAULT_OUTPUT_FILE: &str = "iirs.out";

#[derive(clap::ValueEnum, Debug, Clone, Default, PartialEq)]
pub enum OutputFormat {
    #[default]
    Classic,
    Csv,
    Custom,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmted = match self {
            Self::Classic => "classic",
            Self::Csv => "csv",
            Self::Custom => "custom",
        };
        write!(f, "{fmted}")
    }
}

pub const IUPAC_SYMBOLS: &str = "acgturyswkmbdhvn*-";
#[allow(dead_code)] // used in the tests
pub const ALL_SYMBOLS: &str = "acgturyswkmbdhvn*-$#";
pub const ALL_SYMBOLS_COUNT: usize = 20;
const COMPLEMENT_RULES: [(char, char); 18] = [
    ('a', 't'),
    ('c', 'g'),
    ('g', 'c'),
    ('t', 'a'),
    ('u', 'a'),
    ('r', 'y'),
    ('y', 'r'),
    ('s', 's'),
    ('w', 'w'),
    ('k', 'm'),
    ('m', 'k'),
    ('b', 'v'),
    ('d', 'h'),
    ('h', 'd'),
    ('v', 'b'),
    ('n', 'n'),
    ('*', 'n'),
    ('-', 'n'),
];
const IUPAC_RULES: [(char, &str); 20] = [
    ('a', "a"),
    ('c', "c"),
    ('g', "g"),
    ('t', "t"),
    ('u', "t"),
    ('r', "ag"),
    ('y', "ct"),
    ('s', "gc"),
    ('w', "at"),
    ('k', "gt"),
    ('m', "ac"),
    ('b', "cgt"),
    ('d', "agt"),
    ('h', "act"),
    ('v', "acg"),
    ('n', "acgt"),
    ('*', "acgt"),
    ('-', "acgt"),
    ('$', "$"),
    ('#', "#"),
];

pub fn build_complement_array() -> [u8; 128] {
    let mut complement: [u8; 128] = [0; 128];

    for (key, value) in COMPLEMENT_RULES {
        complement[key as usize] = value as u8;
    }

    complement
}

pub fn build_iupac_rules() -> Vec<(char, BTreeSet<char>)> {
    IUPAC_RULES
        .iter()
        .map(|&(c, t)| (c, t.chars().collect::<BTreeSet<_>>()))
        .collect()
}

mod tests {
    #[test]
    fn test_constants() {
        assert_eq!(super::ALL_SYMBOLS.len(), super::ALL_SYMBOLS_COUNT);
    }
}
