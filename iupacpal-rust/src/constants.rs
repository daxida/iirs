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

pub fn build_complement_array() -> [u8; 128] {
    let mut complement: [u8; 128] = [0; 128];

    for (key, value) in COMPLEMENT_RULES {
        complement[key as usize] = value as u8;
    }

    complement
}

pub fn build_iupac_rules() -> Vec<(char, BTreeSet<char>)> {
    // Vector with the meaning of each IUPAC symbol
    vec![
        ('a', BTreeSet::from(['a'])),
        ('c', BTreeSet::from(['c'])),
        ('g', BTreeSet::from(['g'])),
        ('t', BTreeSet::from(['t'])),
        ('u', BTreeSet::from(['t'])),
        ('r', BTreeSet::from(['a', 'g'])),
        ('y', BTreeSet::from(['c', 't'])),
        ('s', BTreeSet::from(['g', 'c'])),
        ('w', BTreeSet::from(['a', 't'])),
        ('k', BTreeSet::from(['g', 't'])),
        ('m', BTreeSet::from(['a', 'c'])),
        ('b', BTreeSet::from(['c', 'g', 't'])),
        ('d', BTreeSet::from(['a', 'g', 't'])),
        ('h', BTreeSet::from(['a', 'c', 't'])),
        ('v', BTreeSet::from(['a', 'c', 'g'])),
        ('n', BTreeSet::from(['a', 'c', 'g', 't'])),
        ('*', BTreeSet::from(['a', 'c', 'g', 't'])),
        ('-', BTreeSet::from(['a', 'c', 'g', 't'])),
        // Extra cases (separators)
        ('$', BTreeSet::from(['$'])),
        ('#', BTreeSet::from(['#'])),
    ]
}

mod tests {
    #[test]
    fn test_constants() {
        assert_eq!(super::ALL_SYMBOLS.len(), super::ALL_SYMBOLS_COUNT);
    }
}
