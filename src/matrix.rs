use crate::constants::{build_iupac_rules, ALL_SYMBOLS_COUNT};
use std::collections::HashMap;

pub struct MatchMatrix {
    match_matrix: Vec<bool>,    // linearized 1D bool array
    iupac_to_value: Vec<usize>, // used for faster indexing
}

impl MatchMatrix {
    pub fn new() -> Self {
        let iupac_rules = build_iupac_rules();

        // iupac_map is a hash of: (IUPAC_char, set(complements[IUPAC_char]))
        // iupac_to_value is a slice for faster indexing:
        // F.e. ord('$') = 36, which is the 19th key in iupac_rules, so iupac_to_value[36] = 19
        let mut iupac_map = HashMap::new();
        let mut iupac_to_value = vec![0; 128];
        for (index, (iupac_char, mapped_chars)) in iupac_rules.iter().enumerate() {
            iupac_map.insert(*iupac_char, mapped_chars.to_owned());
            iupac_to_value[*iupac_char as usize] = index;
        }

        let mut match_matrix: Vec<bool> = vec![false; ALL_SYMBOLS_COUNT * ALL_SYMBOLS_COUNT];
        for (it1_char, it1_set) in iupac_map.iter() {
            let i = iupac_to_value[*it1_char as usize];
            for (it2_char, it2_set) in iupac_map.iter() {
                let j = iupac_to_value[*it2_char as usize];
                let matching = !it1_set.is_disjoint(it2_set);
                match_matrix[i * ALL_SYMBOLS_COUNT + j] = matching;
            }
        }

        MatchMatrix {
            match_matrix,
            iupac_to_value,
        }
    }

    pub fn match_u8(&self, b1: u8, b2: u8) -> bool {
        let i = self.iupac_to_value[b1 as usize];
        let j = self.iupac_to_value[b2 as usize];
        debug_assert!(i <= 20, "{}", b1);
        debug_assert!(j <= 20, "{}", b2);
        self.match_matrix[i * ALL_SYMBOLS_COUNT + j]
    }
}

pub struct Matcher<'a> {
    mtx: MatchMatrix,
    s: &'a [u8],
}

impl<'a> Matcher<'a> {
    pub fn new(s: &'a [u8]) -> Self {
        Matcher {
            mtx: MatchMatrix::new(),
            s,
        }
    }

    /// Return if the characters at indices i and j match in self.s
    pub fn matches(&self, i: usize, j: usize) -> bool {
        self.mtx.match_u8(self.s[i], self.s[j])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{build_complement_array, ALL_SYMBOLS};

    fn display(matrix: &MatchMatrix, complement: &[u8; 128]) -> String {
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
                // Check for special cases ($ and # behavior) where the complement is not defined
                let matched = if si == '$' || si == '#' || sj == '$' || sj == '#' {
                    false
                } else {
                    matrix.match_u8(si as u8, complement[sj as usize])
                };

                matrix_str += &format!("{} ", if matched { "1" } else { "0" });
            }
            matrix_str += "\n";
        }

        format!("{}{}\n\n", &header, &matrix_str)
    }

    #[test]
    fn test_display() {
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
        let complement = build_complement_array();
        let output = display(&matrix, &complement);

        assert_eq!(output.trim(), expected_output.trim());
    }

    #[test]
    fn test_matches() {
        let matrix = MatchMatrix::new();
        assert!(matrix.match_u8(b'a', b'a'));
        assert!(matrix.match_u8(b'k', b'u'));
        assert!(matrix.match_u8(b'k', b'g'));
    }

    #[test]
    fn test_matches_special_chars() {
        // Because these match, we have to deal with this edge case in test::display
        let matrix = MatchMatrix::new();
        assert!(matrix.match_u8(b'$', b'$'));
        assert!(matrix.match_u8(b'#', b'#'));
    }

    #[test]
    fn test_matches_not() {
        let matrix = MatchMatrix::new();
        assert_eq!(matrix.match_u8(b'a', b't'), false);
    }

    #[test]
    fn test_matches_complementary() {
        let matrix = MatchMatrix::new();
        let complement = build_complement_array();
        assert_eq!(matrix.match_u8(b'k', complement['u' as usize]), false);
    }
}
