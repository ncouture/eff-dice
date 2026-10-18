use crate::dice::DiceRoll;
use crate::error::EffDiceError;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Raw contents of the EFF Large Wordlist embedded from `data/large.txt`.
pub const LARGE_RAW: &str = include_str!("../data/large.txt");
/// Raw contents of the EFF Small Wordlist 1 embedded from `data/small_1.txt`.
pub const SMALL_1_RAW: &str = include_str!("../data/small_1.txt");
/// Raw contents of the EFF Small Wordlist 2 embedded from `data/small_2.txt`.
pub const SMALL_2_RAW: &str = include_str!("../data/small_2.txt");

/// Built-in wordlist variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordListType {
    Large,
    Small1,
    Small2,
}

impl WordListType {
    pub fn dice_per_word(self) -> usize {
        match self {
            WordListType::Large => 5,
            WordListType::Small1 | WordListType::Small2 => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WordListType::Large => "EFF Large Wordlist",
            WordListType::Small1 => "EFF Small Wordlist 1",
            WordListType::Small2 => "EFF Small Wordlist 2",
        }
    }
}

/// A parsed EFF dice wordlist.
#[derive(Debug, Clone)]
pub struct WordList {
    name: String,
    dice_per_word: usize,
    map: HashMap<String, String>,
    entropy_per_word: f64,
}

impl WordList {
    /// Loads the standard EFF Large Wordlist embedded from `data/large.txt` (5 dice, 7776 words).
    pub fn large() -> Self {
        Self::from_tsv(LARGE_RAW, 5, "EFF Large Wordlist")
            .expect("Embedded data/large.txt is valid")
    }

    /// Loads the EFF Small Wordlist 1 embedded from `data/small_1.txt` (4 dice, 1296 short words).
    pub fn small_1() -> Self {
        Self::from_tsv(SMALL_1_RAW, 4, "EFF Small Wordlist 1")
            .expect("Embedded data/small_1.txt is valid")
    }

    /// Loads the EFF Small Wordlist 2 embedded from `data/small_2.txt` (4 dice, 1296 memorable words).
    pub fn small_2() -> Self {
        Self::from_tsv(SMALL_2_RAW, 4, "EFF Small Wordlist 2")
            .expect("Embedded data/small_2.txt is valid")
    }

    /// Creates a wordlist by type.
    pub fn from_type(wl_type: WordListType) -> Self {
        match wl_type {
            WordListType::Large => Self::large(),
            WordListType::Small1 => Self::small_1(),
            WordListType::Small2 => Self::small_2(),
        }
    }

    /// Parses TSV / whitespace-separated content: `<roll_digits>\t<word>`.
    pub fn from_tsv(content: &str, dice_per_word: usize, name: &str) -> Result<Self, EffDiceError> {
        let mut map = HashMap::new();
        for (line_idx, line) in content.lines().enumerate() {
            let line_number = line_idx + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Split by tab or whitespace (roll <whitespace> word)
            let (roll, word) = match trimmed.split_once(|c: char| c.is_whitespace()) {
                Some((r, w)) => (r.trim(), w.trim()),
                None => {
                    return Err(EffDiceError::WordListParseError {
                        line_number,
                        details: format!("Missing word for roll '{trimmed}'"),
                    });
                }
            };

            // Validate roll
            let parsed_roll = match DiceRoll::parse(roll, Some(dice_per_word)) {
                Ok(r) => r,
                Err(err) => {
                    return Err(EffDiceError::WordListParseError {
                        line_number,
                        details: format!("Invalid roll '{roll}': {err}"),
                    });
                }
            };
            map.insert(parsed_roll.as_str().to_string(), word.to_string());
        }

        if map.is_empty() {
            return Err(EffDiceError::EmptyWordList);
        }

        let entropy_per_word = (map.len() as f64).log2();
        Ok(Self {
            name: name.to_string(),
            dice_per_word,
            map,
            entropy_per_word,
        })
    }

    /// Parses words-only content (one word per line), synthesizing the base-6 dice rolls.
    pub fn from_words_only(
        content: &str,
        dice_per_word: usize,
        name: &str,
    ) -> Result<Self, EffDiceError> {
        let mut map = HashMap::new();
        let mut index = 0;
        let max_capacity = 6usize.pow(dice_per_word as u32);

        for (line_idx, line) in content.lines().enumerate() {
            let word = line.trim();
            if word.is_empty() || word.starts_with('#') {
                continue;
            }
            if index >= max_capacity {
                return Err(EffDiceError::WordListParseError {
                    line_number: line_idx + 1,
                    details: format!(
                        "Too many words for {dice_per_word} dice (max capacity is {max_capacity})"
                    ),
                });
            }
            let roll_str = Self::index_to_roll(index, dice_per_word)?;
            map.insert(roll_str, word.to_string());
            index += 1;
        }

        if map.is_empty() {
            return Err(EffDiceError::EmptyWordList);
        }

        let entropy_per_word = (map.len() as f64).log2();
        Ok(Self {
            name: name.to_string(),
            dice_per_word,
            map,
            entropy_per_word,
        })
    }

    /// Converts a 0-based sequential index into a base-6 dice roll string (digits '1'..='6').
    pub fn index_to_roll(index: usize, dice_per_word: usize) -> Result<String, EffDiceError> {
        let max_capacity = 6usize.pow(dice_per_word as u32);
        if index >= max_capacity {
            return Err(EffDiceError::WordListParseError {
                line_number: index + 1,
                details: format!(
                    "Index {index} exceeds maximum roll capacity {max_capacity} for {dice_per_word} dice"
                ),
            });
        }

        let mut rem = index;
        let mut chars = vec!['1'; dice_per_word];
        for i in (0..dice_per_word).rev() {
            let digit = (rem % 6) as u8 + 1;
            chars[i] = (b'0' + digit) as char;
            rem /= 6;
        }

        Ok(chars.into_iter().collect())
    }

    /// Loads a wordlist from a file path. Auto-detects whether the file is TSV or words-only.
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        explicit_dice_per_word: Option<usize>,
    ) -> Result<Self, EffDiceError> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref)?;
        let file_name = path_ref
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Custom Wordlist");

        // Peek at first non-empty line
        let first_line = content
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && !l.starts_with('#'));

        let is_tsv = match first_line {
            Some(l) => {
                let parts: Vec<&str> = l.split_whitespace().collect();
                parts.len() >= 2 && parts[0].chars().all(|c| ('1'..='6').contains(&c))
            }
            None => return Err(EffDiceError::EmptyWordList),
        };

        if is_tsv {
            let first_token = first_line.unwrap().split_whitespace().next().unwrap();
            let detected_dpw = first_token.len();
            let dice_per_word = explicit_dice_per_word.unwrap_or(detected_dpw);
            Self::from_tsv(&content, dice_per_word, file_name)
        } else {
            // Words-only list
            let word_count = content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .count();

            let dice_per_word = if let Some(dpw) = explicit_dice_per_word {
                dpw
            } else if word_count == 7776 {
                5
            } else if word_count == 1296 {
                4
            } else {
                // Determine minimum dice needed
                let mut dpw = 1;
                while 6usize.pow(dpw as u32) < word_count {
                    dpw += 1;
                }
                dpw
            };

            Self::from_words_only(&content, dice_per_word, file_name)
        }
    }

    /// Retrieves the word corresponding to a given roll string (e.g. "11111").
    pub fn get(&self, roll: &str) -> Option<&str> {
        self.map.get(roll).map(|s| s.as_str())
    }

    /// Looks up a word for a `DiceRoll` struct, returning an error if not found.
    pub fn lookup(&self, roll: &DiceRoll) -> Result<&str, EffDiceError> {
        if roll.len() != self.dice_per_word {
            return Err(EffDiceError::InvalidRollLength {
                expected: self.dice_per_word,
                found: roll.len(),
                roll: roll.as_str().to_string(),
            });
        }
        self.get(roll.as_str())
            .ok_or_else(|| EffDiceError::RollNotFound(roll.as_str().to_string()))
    }

    /// Returns the name of the wordlist.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of dice required per word.
    pub fn dice_per_word(&self) -> usize {
        self.dice_per_word
    }

    /// Returns the total number of words in this list.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the wordlist contains no words.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the entropy (in bits) provided by each word from this list.
    pub fn entropy_per_word(&self) -> f64 {
        self.entropy_per_word
    }

    /// Calculates total entropy (in bits) for a passphrase of `word_count` words.
    pub fn total_entropy(&self, word_count: usize) -> f64 {
        word_count as f64 * self.entropy_per_word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_wordlist_embedded() {
        let wl = WordList::large();
        assert_eq!(wl.len(), 7776);
        assert_eq!(wl.dice_per_word(), 5);
        assert_eq!(wl.get("11111"), Some("abacus"));
        assert_eq!(wl.get("66666"), Some("zoom"));
        // Check entropy: log2(7776) ≈ 12.9248 bits
        assert!((wl.entropy_per_word() - 12.9248).abs() < 0.01);
    }

    #[test]
    fn test_small_1_wordlist_embedded() {
        let wl = WordList::small_1();
        assert_eq!(wl.len(), 1296);
        assert_eq!(wl.dice_per_word(), 4);
        assert_eq!(wl.get("1111"), Some("acid"));
        assert_eq!(wl.get("6666"), Some("zoom"));
        // Check entropy: log2(1296) ≈ 10.3398 bits
        assert!((wl.entropy_per_word() - 10.3398).abs() < 0.01);
    }

    #[test]
    fn test_small_2_wordlist_embedded() {
        let wl = WordList::small_2();
        assert_eq!(wl.len(), 1296);
        assert_eq!(wl.dice_per_word(), 4);
        assert_eq!(wl.get("1111"), Some("aardvark"));
        assert_eq!(wl.get("6666"), Some("zucchini"));
    }

    #[test]
    fn test_index_to_roll() {
        assert_eq!(WordList::index_to_roll(0, 5).unwrap(), "11111");
        assert_eq!(WordList::index_to_roll(1, 5).unwrap(), "11112");
        assert_eq!(WordList::index_to_roll(5, 5).unwrap(), "11116");
        assert_eq!(WordList::index_to_roll(6, 5).unwrap(), "11121");
        assert_eq!(WordList::index_to_roll(7775, 5).unwrap(), "66666");

        assert_eq!(WordList::index_to_roll(0, 4).unwrap(), "1111");
        assert_eq!(WordList::index_to_roll(1295, 4).unwrap(), "6666");
    }

    #[test]
    fn test_words_only_parsing() {
        let content = "apple\nbanana\ncherry\ndate";
        let wl = WordList::from_words_only(content, 1, "test").unwrap();
        assert_eq!(wl.len(), 4);
        assert_eq!(wl.get("1"), Some("apple"));
        assert_eq!(wl.get("2"), Some("banana"));
        assert_eq!(wl.get("3"), Some("cherry"));
        assert_eq!(wl.get("4"), Some("date"));
    }

    #[test]
    fn test_lookup_dice_roll() {
        let wl = WordList::large();
        let roll = DiceRoll::parse("11111", Some(5)).unwrap();
        assert_eq!(wl.lookup(&roll).unwrap(), "abacus");

        let bad_roll = DiceRoll::parse("1111", Some(4)).unwrap();
        assert!(matches!(
            wl.lookup(&bad_roll),
            Err(EffDiceError::InvalidRollLength { .. })
        ));
    }
}
