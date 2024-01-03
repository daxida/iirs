use std::collections::{BTreeSet, HashMap};

const ALL_SYMBOLS: &str = "acgturyswkmbdhvn*-$#";
const ALL_SYMBOLS_COUNT: usize = 20;

#[derive(Clone, Debug)]
pub struct MatchMatrix {
    match_matrix: Vec<bool>,    // 1D-array of bool for faster access (than 2D)
    iupac_to_value: Vec<usize>, // used for faster indexing
}

impl MatchMatrix {
    /// Doesn't need complements!!!
    pub fn new() -> Self {
        // Vector with the meaning of each IUPAC symbol
        let iupac_rules: Vec<(char, BTreeSet<char>)> = vec![
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
        ];

        // Builds iupac_map & iupac_to_value
        // iupac_map is a hash of: (IUPAC_char, set(complements[IUPAC_char]))
        // iupac_to_value is a slice for faster indexing:
        // F.e. ord('$') = 36, which is the 19th key in iupac_rules, so iupac_to_value[36] = 19
        let mut iupac_map = HashMap::new();
        let mut iupac_to_value: Vec<usize> = vec![64; 128]; // 64 = @
        for (index, (iupac_char, mapped_chars)) in iupac_rules.iter().enumerate() {
            iupac_map.insert(*iupac_char, mapped_chars.to_owned());
            iupac_to_value[*iupac_char as usize] = index;
        }
        // dbg!(&iupac_map);
        // dbg!(&iupac_to_value);

        // Build match matrix
        let mut match_matrix: Vec<bool> = vec![false; ALL_SYMBOLS_COUNT * ALL_SYMBOLS_COUNT];
        for (it1_char, it1_set) in iupac_map.iter() {
            let i = iupac_to_value[*it1_char as usize];

            for (it2_char, it2_set) in iupac_map.iter() {
                let j = iupac_to_value[*it2_char as usize];
                // let mut matching = false;
                // for it_set1 in it1_set {
                //     for it_set2 in it2_set {
                //         if it_set1 == it_set2 {
                //             matching = true;
                //             break;
                //         }
                //     }
                // }
                let matching = !it1_set.is_disjoint(it2_set);
                match_matrix[i + i * (ALL_SYMBOLS_COUNT - 1) + j] = matching;
            }
        }

        MatchMatrix {
            match_matrix,
            iupac_to_value,
        }
    }

    // Match function
    pub fn match_chars(&self, ch1: char, ch2: char) -> bool {
        let i = self.iupac_to_value[ch1 as usize];
        let j = self.iupac_to_value[ch2 as usize];
        assert!(i <= 20, "{}", ch1);
        assert!(j <= 20, "{}", ch2);
        self.match_matrix[i + i * (ALL_SYMBOLS_COUNT - 1) + j]
    }

    // Optionally print match matrix
    #[allow(dead_code)]
    pub fn display(&self, complement: &[char; 128]) -> String {
        let header = format!(
            "Match Matrix:\n  {}\n",
            ALL_SYMBOLS
                .chars()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        let mut matrix_str = String::new();
        for si in ALL_SYMBOLS.chars() {
            matrix_str += &format!("{} ", si);
            for sj in ALL_SYMBOLS.chars() {
                // Check for special cases ($ and # behavior)
                let matched = if si == '$' || si == '#' || sj == '$' || sj == '#' {
                    false
                } else {
                    self.match_chars(si, complement[sj as usize])
                };

                matrix_str += &format!("{} ", if matched { "1" } else { "0" });
            }
            matrix_str += "\n";
        }

        format!("{}{}\n\n", &header, &matrix_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_complement() -> [char; 128] {
        let complement_rules = vec![
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

        let mut complement: [char; 128] = ['@'; 128];
        for (key, value) in complement_rules {
            complement[key as usize] = value
        }

        complement
    }

    #[test]
    fn test_constants() {
        assert_eq!(ALL_SYMBOLS.len(), ALL_SYMBOLS_COUNT);
    }

    #[test]
    fn test_display() {
        // Populate with your expected string
        let expected_output = "Match Matrix:\n  \
              a c g t u r y s w k m b d h v n * - $ #\n\
            a 0 0 0 1 1 0 1 0 1 1 0 1 1 1 0 1 1 1 0 0 \n\
            c 0 0 1 0 0 1 0 1 0 1 0 1 1 0 1 1 1 1 0 0 \n\
            g 0 1 0 0 0 0 1 1 0 0 1 1 0 1 1 1 1 1 0 0 \n\
            t 1 0 0 0 0 1 0 0 1 0 1 0 1 1 1 1 1 1 0 0 \n\
            u 1 0 0 0 0 1 0 0 1 0 1 0 1 1 1 1 1 1 0 0 \n\
            r 0 1 0 1 1 0 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            y 1 0 1 0 0 1 0 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            s 0 1 1 0 0 1 1 1 0 1 1 1 1 1 1 1 1 1 0 0 \n\
            w 1 0 0 1 1 1 1 0 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            k 1 1 0 0 0 1 1 1 1 0 1 1 1 1 1 1 1 1 0 0 \n\
            m 0 0 1 1 1 1 1 1 1 1 0 1 1 1 1 1 1 1 0 0 \n\
            b 1 1 1 0 0 1 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            d 1 1 0 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            h 1 0 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            v 0 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            n 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            * 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            - 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 0 0 \n\
            $ 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 \n\
            # 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";

        let matrix = MatchMatrix::new();
        let complement = get_complement();
        let output = matrix.display(&complement);

        assert_eq!(output.trim(), expected_output.trim());
    }

    #[test]
    fn test_matches() {
        let matrix = MatchMatrix::new();
        assert_eq!(matrix.match_chars('a', 'a'), true);
        assert_eq!(matrix.match_chars('k', 'u'), true);
        assert_eq!(matrix.match_chars('k', 'g'), true);
    }

    #[test]
    fn test_matches_special_chars() {
        let matrix = MatchMatrix::new();
        assert_eq!(matrix.match_chars('$', '$'), true);
        assert_eq!(matrix.match_chars('#', '#'), true);
    }

    #[test]
    fn test_matches_not() {
        let matrix = MatchMatrix::new();
        assert_eq!(matrix.match_chars('a', 't'), false);
    }

    #[test]
    fn test_matches_complementary() {
        let matrix = MatchMatrix::new();
        let complement = get_complement();
        assert_eq!(matrix.match_chars('k', complement['u' as usize]), false);
    }
}
