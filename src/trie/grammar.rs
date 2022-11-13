use std::collections::HashMap;
use std::fmt;

/// You know what this means...
#[derive(Debug, Copy, Clone)]
pub enum Case {
    Sensitive,
    Insensitive
}

/// This is the set of possible chars in the trie data structure.
#[derive(Debug, Clone)]
pub struct Grammar {
    mapping: HashMap<char, usize>,
    sense: Case
}

impl Grammar {
    pub fn to_indices(&self, s: &str) -> Result<Vec<usize>, String> {
        let mut out = vec![];

        for raw_char in s.chars() {
            match self.idx(raw_char) {
                None => {
                    return Err(format!("char '{}' is not part of grammar: {}", raw_char, self));
                },
                Some(i) => {
                    out.push(i);
                }
            }
        }

        Ok(out)
    }

    pub fn from(s_slice: &str, sense: Case) -> Self {
        let mut chars: Vec<char> = s_slice.chars().collect();
        chars.sort_by(|a, b| b.cmp(a));

        let mut mapping = HashMap::new();

        chars.iter().for_each(
            |c| {
                let k = preprocess_char(c, &sense);
                if !mapping.contains_key(&k) {
                    let idx = mapping.len();
                    mapping.insert(k, idx);
                }
            }
        );

        Grammar { mapping, sense }
    }

    pub fn idx(&self, c: char) -> Option<usize> {
        self.mapping.get(&preprocess_char(&c, &self.sense)).cloned()
    }

    pub fn seq(&self) -> Vec<char> {
        let mut seq = vec!['$'; self.mapping.len()];
        self.mapping.iter().for_each(
            |(k, v)| {
                seq[*v] = k.clone();
            }
        );
        seq
    }
}

impl Default for Grammar {
    fn default() -> Self {
        Grammar::from(&"abcdefghijklmnopqrstuvwxyz", Case::Insensitive)
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.seq())
    }
}

fn preprocess_char(c: &char, sense: &Case) -> char {
    match sense {
        Case::Sensitive => {
            c.clone()
        }
        Case::Insensitive => {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else {
                c.clone()
            }
        }
    }
}
