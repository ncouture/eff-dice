use crate::dice::DiceRoll;
use crate::error::EffDiceError;
use crate::wordlist::WordList;
use rand::Rng;
use std::fmt;

/// Text casing options for words in a passphrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordTransform {
    #[default]
    None,
    TitleCase,
    UpperCase,
    LowerCase,
}

impl WordTransform {
    pub fn apply(&self, word: &str) -> String {
        match self {
            WordTransform::None => word.to_string(),
            WordTransform::LowerCase => word.to_lowercase(),
            WordTransform::UpperCase => word.to_uppercase(),
            WordTransform::TitleCase => {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let mut s = first.to_uppercase().collect::<String>();
                        s.push_str(&chars.as_str().to_lowercase());
                        s
                    }
                }
            }
        }
    }
}

/// Represents a generated or constructed passphrase.
#[derive(Debug, Clone, PartialEq)]
pub struct Passphrase {
    words: Vec<String>,
    rolls: Vec<DiceRoll>,
    delimiter: String,
    entropy_bits: f64,
}

impl Passphrase {
    /// Creates a new `Passphrase`.
    pub fn new(
        words: Vec<String>,
        rolls: Vec<DiceRoll>,
        delimiter: String,
        entropy_bits: f64,
    ) -> Self {
        Self {
            words,
            rolls,
            delimiter,
            entropy_bits,
        }
    }

    /// Returns the combined passphrase text.
    pub fn text(&self) -> String {
        self.words.join(&self.delimiter)
    }

    /// Returns a slice of individual words.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// Returns the number of words in this passphrase.
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Returns a slice of the dice rolls corresponding to each word.
    pub fn rolls(&self) -> &[DiceRoll] {
        &self.rolls
    }

    /// Returns the delimiter between words.
    pub fn delimiter(&self) -> &str {
        &self.delimiter
    }

    /// Returns the total entropy of the passphrase in bits.
    pub fn entropy(&self) -> f64 {
        self.entropy_bits
    }

    /// Returns the total number of dice used.
    pub fn total_dice(&self) -> usize {
        self.rolls.iter().map(|r| r.len()).sum()
    }

    /// Formats a detailed summary of the passphrase, including dice rolls and entropy.
    pub fn detailed_summary(&self) -> String {
        let roll_strs: Vec<&str> = self.rolls.iter().map(|r| r.as_str()).collect();
        let total_dice = self.total_dice();
        let word_count = self.words.len();
        let bits_per_word = if word_count > 0 {
            self.entropy_bits / word_count as f64
        } else {
            0.0
        };

        let mut lines = Vec::new();
        lines.push(format!("Passphrase:    {}", self.text()));
        lines.push(format!("Word count:    {word_count}"));
        lines.push(format!(
            "Dice rolls:    {} ({total_dice} dice)",
            roll_strs.join(" ")
        ));
        lines.push(format!(
            "Entropy:       {:.2} bits (~{:.2} bits/word)",
            self.entropy_bits, bits_per_word
        ));

        lines.push(String::new());
        lines.push("Roll Breakdown:".to_string());
        for (i, (roll, word)) in self.rolls.iter().zip(self.words.iter()).enumerate() {
            lines.push(format!("  {:2}. {} -> {}", i + 1, roll.as_str(), word));
        }

        lines.join("\n")
    }
}

impl fmt::Display for Passphrase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// Core generator for EFF dice passphrases.
pub struct PassphraseGenerator;

impl PassphraseGenerator {
    /// Generates a passphrase by specifying the total number of dice to roll.
    ///
    /// # Errors
    /// - `EffDiceError::ZeroDice` if `dice_count == 0`
    /// - `EffDiceError::InvalidDiceCount` if `dice_count` is not a multiple of `wordlist.dice_per_word()`
    pub fn generate_from_dice_count<R: Rng + ?Sized>(
        wordlist: &WordList,
        dice_count: usize,
        delimiter: &str,
        transform: WordTransform,
        rng: &mut R,
    ) -> Result<Passphrase, EffDiceError> {
        if dice_count == 0 {
            return Err(EffDiceError::ZeroDice);
        }

        let dpw = wordlist.dice_per_word();
        if !dice_count.is_multiple_of(dpw) {
            let suggested_lower = (dice_count / dpw) * dpw;
            let suggested_higher = suggested_lower + dpw;
            return Err(EffDiceError::InvalidDiceCount {
                dice_count,
                dice_per_word: dpw,
                suggested_lower,
                suggested_higher,
            });
        }

        let word_count = dice_count / dpw;
        Self::generate_from_word_count(wordlist, word_count, delimiter, transform, rng)
    }

    /// Generates a passphrase by specifying the number of words.
    ///
    /// # Errors
    /// - `EffDiceError::ZeroWords` if `word_count == 0`
    pub fn generate_from_word_count<R: Rng + ?Sized>(
        wordlist: &WordList,
        word_count: usize,
        delimiter: &str,
        transform: WordTransform,
        rng: &mut R,
    ) -> Result<Passphrase, EffDiceError> {
        if word_count == 0 {
            return Err(EffDiceError::ZeroWords);
        }

        let dpw = wordlist.dice_per_word();
        let mut words = Vec::with_capacity(word_count);
        let mut rolls = Vec::with_capacity(word_count);

        for _ in 0..word_count {
            let roll = DiceRoll::roll(dpw, rng);
            let word = wordlist.lookup(&roll)?;
            words.push(transform.apply(word));
            rolls.push(roll);
        }

        let entropy_bits = wordlist.total_entropy(word_count);
        Ok(Passphrase::new(
            words,
            rolls,
            delimiter.to_string(),
            entropy_bits,
        ))
    }

    /// Generates multiple passphrases.
    pub fn generate_multiple<R: Rng + ?Sized>(
        count: usize,
        wordlist: &WordList,
        dice_count: Option<usize>,
        word_count: Option<usize>,
        delimiter: &str,
        transform: WordTransform,
        rng: &mut R,
    ) -> Result<Vec<Passphrase>, EffDiceError> {
        if count == 0 {
            return Ok(Vec::new());
        }

        let resolved_word_count = match (dice_count, word_count) {
            (Some(d), Some(w)) => {
                let dpw = wordlist.dice_per_word();
                if d != w * dpw {
                    return Err(EffDiceError::ConflictingDiceAndWordCount {
                        dice_count: d,
                        word_count: w,
                        dice_per_word: dpw,
                    });
                }
                w
            }
            (Some(d), None) => {
                let dpw = wordlist.dice_per_word();
                if d == 0 {
                    return Err(EffDiceError::ZeroDice);
                }
                if !d.is_multiple_of(dpw) {
                    let suggested_lower = (d / dpw) * dpw;
                    let suggested_higher = suggested_lower + dpw;
                    return Err(EffDiceError::InvalidDiceCount {
                        dice_count: d,
                        dice_per_word: dpw,
                        suggested_lower,
                        suggested_higher,
                    });
                }
                d / dpw
            }
            (None, Some(w)) => {
                if w == 0 {
                    return Err(EffDiceError::ZeroWords);
                }
                w
            }
            (None, None) => {
                // Default to 5 words for large, 6 for small
                if wordlist.dice_per_word() == 5 {
                    5
                } else {
                    6
                }
            }
        };

        let mut passphrases = Vec::with_capacity(count);
        for _ in 0..count {
            passphrases.push(Self::generate_from_word_count(
                wordlist,
                resolved_word_count,
                delimiter,
                transform,
                rng,
            )?);
        }

        Ok(passphrases)
    }

    /// Builds a passphrase based on user-provided manual dice rolls.
    pub fn from_manual_rolls(
        wordlist: &WordList,
        rolls: &[DiceRoll],
        delimiter: &str,
        transform: WordTransform,
    ) -> Result<Passphrase, EffDiceError> {
        if rolls.is_empty() {
            return Err(EffDiceError::ZeroWords);
        }

        let mut words = Vec::with_capacity(rolls.len());
        for roll in rolls {
            let word = wordlist.lookup(roll)?;
            words.push(transform.apply(word));
        }

        let entropy_bits = wordlist.total_entropy(rolls.len());
        Ok(Passphrase::new(
            words,
            rolls.to_vec(),
            delimiter.to_string(),
            entropy_bits,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_generate_from_dice_count() {
        let wl = WordList::large();
        let mut rng = StdRng::seed_from_u64(999);

        let pass = PassphraseGenerator::generate_from_dice_count(
            &wl,
            25,
            " ",
            WordTransform::None,
            &mut rng,
        )
        .unwrap();

        assert_eq!(pass.words().len(), 5);
        assert_eq!(pass.word_count(), 5);
        assert_eq!(pass.rolls().len(), 5);
        assert_eq!(pass.total_dice(), 25);
        assert!((pass.entropy() - (5.0 * 12.9248)).abs() < 0.1);
    }

    #[test]
    fn test_invalid_dice_count_error() {
        let wl = WordList::large();
        let mut rng = StdRng::seed_from_u64(999);

        let err = PassphraseGenerator::generate_from_dice_count(
            &wl,
            23,
            " ",
            WordTransform::None,
            &mut rng,
        )
        .unwrap_err();

        match err {
            EffDiceError::InvalidDiceCount {
                dice_count,
                dice_per_word,
                suggested_lower,
                suggested_higher,
            } => {
                assert_eq!(dice_count, 23);
                assert_eq!(dice_per_word, 5);
                assert_eq!(suggested_lower, 20);
                assert_eq!(suggested_higher, 25);
            }
            _ => panic!("Expected InvalidDiceCount error"),
        }
    }

    #[test]
    fn test_zero_dice_error() {
        let wl = WordList::large();
        let mut rng = StdRng::seed_from_u64(999);

        let err = PassphraseGenerator::generate_from_dice_count(
            &wl,
            0,
            " ",
            WordTransform::None,
            &mut rng,
        )
        .unwrap_err();

        assert_eq!(err, EffDiceError::ZeroDice);
    }

    #[test]
    fn test_generate_multiple() {
        let wl = WordList::large();
        let mut rng = StdRng::seed_from_u64(100);

        let passes = PassphraseGenerator::generate_multiple(
            3,
            &wl,
            Some(20),
            None,
            "-",
            WordTransform::TitleCase,
            &mut rng,
        )
        .unwrap();

        assert_eq!(passes.len(), 3);
        for p in &passes {
            assert_eq!(p.words().len(), 4);
            assert_eq!(p.delimiter(), "-");
            assert!(p.text().contains('-'));
            // Check that first char of each word is uppercase
            for w in p.words() {
                assert!(w.chars().next().unwrap().is_uppercase());
            }
        }
    }

    #[test]
    fn test_conflicting_dice_and_word_count() {
        let wl = WordList::large();
        let mut rng = StdRng::seed_from_u64(100);

        let err = PassphraseGenerator::generate_multiple(
            1,
            &wl,
            Some(20),
            Some(5), // 20 dice != 5 * 5 dice
            " ",
            WordTransform::None,
            &mut rng,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            EffDiceError::ConflictingDiceAndWordCount { .. }
        ));
    }

    #[test]
    fn test_from_manual_rolls() {
        let wl = WordList::large();
        let r1 = DiceRoll::parse("11111", Some(5)).unwrap();
        let r2 = DiceRoll::parse("11112", Some(5)).unwrap();
        let r3 = DiceRoll::parse("66666", Some(5)).unwrap();

        let pass =
            PassphraseGenerator::from_manual_rolls(&wl, &[r1, r2, r3], " ", WordTransform::None)
                .unwrap();

        assert_eq!(pass.text(), "abacus abdomen zoom");
        assert_eq!(pass.total_dice(), 15);
    }

    #[test]
    fn test_detailed_summary() {
        let wl = WordList::large();
        let r1 = DiceRoll::parse("11111", Some(5)).unwrap();
        let pass =
            PassphraseGenerator::from_manual_rolls(&wl, &[r1], " ", WordTransform::None).unwrap();

        let summary = pass.detailed_summary();
        assert!(summary.contains("Passphrase:    abacus"));
        assert!(summary.contains("Dice rolls:    11111 (5 dice)"));
        assert!(summary.contains("1. 11111 -> abacus"));
    }
}
