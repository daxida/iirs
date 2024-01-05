use std::collections::BTreeSet;

pub const IUPAC_SYMBOLS: &str = "acgturyswkmbdhvn*-";
#[allow(dead_code)] // used in the tests
pub const ALL_SYMBOLS: &str = "acgturyswkmbdhvn*-$#";
pub const ALL_SYMBOLS_COUNT: usize = 20;
pub const COMPLEMENT_RULES: [(char, char); 18] = [
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
