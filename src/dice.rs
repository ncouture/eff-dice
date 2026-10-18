use crate::error::EffDiceError;
use rand::Rng;
use std::fmt;

/// Represents a single 6-sided die roll result (1 to 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Die(u8);

impl Die {
    /// Creates a new `Die` with the specified value (1 to 6).
    pub fn new(value: u8) -> Result<Self, EffDiceError> {
        if (1..=6).contains(&value) {
            Ok(Self(value))
        } else {
            let ch = char::from_digit(value as u32, 10).unwrap_or('?');
            Err(EffDiceError::InvalidDieDigit(ch))
        }
    }

    /// Parses a die face from an ASCII char ('1'..='6').
    pub fn from_char(c: char) -> Result<Self, EffDiceError> {
        match c {
            '1'..='6' => Ok(Self((c as u8) - b'0')),
            _ => Err(EffDiceError::InvalidDieDigit(c)),
        }
    }

    /// Rolls a single random die using the provided random number generator.
    pub fn roll<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let value: u8 = rng.random_range(1..=6);
        Self(value)
    }

    /// Returns the integer value of the die (1 to 6).
    pub fn value(self) -> u8 {
        self.0
    }

    /// Returns the character representation ('1' to '6').
    pub fn as_char(self) -> char {
        (b'0' + self.0) as char
    }
}

impl fmt::Display for Die {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a roll of one or more dice (e.g., 5 dice like "11111" or 4 dice like "1111").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiceRoll {
    dice: Vec<Die>,
    string_repr: String,
}

impl DiceRoll {
    /// Constructs a `DiceRoll` from a vector of `Die`.
    pub fn from_dice(dice: Vec<Die>) -> Self {
        let string_repr: String = dice.iter().map(|d| d.as_char()).collect();
        Self { dice, string_repr }
    }

    /// Parses a `DiceRoll` from a string slice (e.g. "11111").
    /// If `expected_len` is provided, ensures that exactly that many dice are present.
    pub fn parse(s: &str, expected_len: Option<usize>) -> Result<Self, EffDiceError> {
        let trimmed = s.trim();
        let mut dice = Vec::with_capacity(trimmed.len());
        for c in trimmed.chars() {
            dice.push(Die::from_char(c)?);
        }

        if let Some(expected) = expected_len {
            if dice.len() != expected {
                return Err(EffDiceError::InvalidRollLength {
                    expected,
                    found: dice.len(),
                    roll: trimmed.to_string(),
                });
            }
        }

        let string_repr = trimmed.to_string();
        Ok(Self { dice, string_repr })
    }

    /// Rolls `count` dice using the provided RNG.
    pub fn roll<R: Rng + ?Sized>(count: usize, rng: &mut R) -> Self {
        let mut dice = Vec::with_capacity(count);
        let mut string_repr = String::with_capacity(count);
        for _ in 0..count {
            let die = Die::roll(rng);
            string_repr.push(die.as_char());
            dice.push(die);
        }
        Self { dice, string_repr }
    }

    /// Returns the string representation of the roll, e.g. "11111".
    pub fn as_str(&self) -> &str {
        &self.string_repr
    }

    /// Returns the individual dice in this roll.
    pub fn dice(&self) -> &[Die] {
        &self.dice
    }

    /// Returns the number of dice in this roll.
    pub fn len(&self) -> usize {
        self.dice.len()
    }

    /// Returns true if there are no dice in this roll.
    pub fn is_empty(&self) -> bool {
        self.dice.is_empty()
    }

    /// Parses a sequence of rolls from input text.
    /// Supports:
    /// - Delimited rolls: "11111 23456 66666" or "11111,23456,66666"
    /// - Individual spaced dice: "1 1 1 1 1  2 3 4 5 6" or "1, 1, 1, 1, 1"
    /// - Continuous stream of digits: "111112345666666" (split by `dice_per_word`)
    pub fn parse_sequence(input: &str, dice_per_word: usize) -> Result<Vec<Self>, EffDiceError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(EffDiceError::InvalidRollInput(
                "Input string is empty".to_string(),
            ));
        }

        // Split tokens by whitespace or common delimiters
        let tokens: Vec<&str> = trimmed
            .split(|c: char| {
                c.is_whitespace()
                    || c == ','
                    || c == ';'
                    || c == '-'
                    || c == '|'
                    || c == '/'
                    || c == ':'
            })
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.is_empty() {
            return Err(EffDiceError::InvalidRollInput(
                "Input string contains no roll tokens".to_string(),
            ));
        }

        // Case 1: Every token is already exactly `dice_per_word` characters
        if tokens.iter().all(|t| t.len() == dice_per_word) {
            let mut rolls = Vec::with_capacity(tokens.len());
            for token in tokens {
                let roll = Self::parse(token, Some(dice_per_word))?;
                rolls.push(roll);
            }
            return Ok(rolls);
        }

        // Case 2: All tokens are single characters (e.g. individual dice: "1 1 1 1 1  2 2 2 2 2")
        if tokens.iter().all(|t| t.len() == 1) {
            let mut dice = Vec::with_capacity(tokens.len());
            for token in &tokens {
                let ch = token.chars().next().unwrap();
                dice.push(Die::from_char(ch)?);
            }

            if !dice.len().is_multiple_of(dice_per_word) {
                return Err(EffDiceError::InvalidRollLength {
                    expected: dice_per_word,
                    found: dice.len(),
                    roll: trimmed.to_string(),
                });
            }

            let num_words = dice.len() / dice_per_word;
            let mut rolls = Vec::with_capacity(num_words);
            for chunk in dice.chunks(dice_per_word) {
                rolls.push(Self::from_dice(chunk.to_vec()));
            }
            return Ok(rolls);
        }

        // Case 3: Exactly one token with no separators (continuous digit stream)
        if tokens.len() == 1 {
            let token = tokens[0];
            if !token.len().is_multiple_of(dice_per_word) {
                return Err(EffDiceError::InvalidRollInput(format!(
                    "Continuous roll string \"{token}\" length ({}) is not a multiple of {} dice per word.",
                    token.len(),
                    dice_per_word
                )));
            }

            let num_words = token.len() / dice_per_word;
            let mut rolls = Vec::with_capacity(num_words);
            for i in 0..num_words {
                let chunk = &token[i * dice_per_word..(i + 1) * dice_per_word];
                let roll = Self::parse(chunk, Some(dice_per_word))?;
                rolls.push(roll);
            }
            return Ok(rolls);
        }

        // Case 4: Inconsistent token lengths (e.g. "11111 2222")
        if let Some(mismatched) = tokens.iter().find(|t| t.len() != dice_per_word) {
            return Err(EffDiceError::InvalidRollLength {
                expected: dice_per_word,
                found: mismatched.len(),
                roll: mismatched.to_string(),
            });
        }

        Err(EffDiceError::InvalidRollInput(format!(
            "Unable to parse roll sequence from \"{trimmed}\""
        )))
    }
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string_repr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_die_creation_and_bounds() {
        for v in 1..=6 {
            let die = Die::new(v).unwrap();
            assert_eq!(die.value(), v);
            assert_eq!(die.as_char(), (b'0' + v) as char);
            assert_eq!(Die::from_char((b'0' + v) as char).unwrap(), die);
        }

        assert!(matches!(
            Die::new(0),
            Err(EffDiceError::InvalidDieDigit('0'))
        ));
        assert!(matches!(
            Die::new(7),
            Err(EffDiceError::InvalidDieDigit('7'))
        ));
        assert!(matches!(
            Die::from_char('a'),
            Err(EffDiceError::InvalidDieDigit('a'))
        ));
    }

    #[test]
    fn test_die_roll() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..100 {
            let die = Die::roll(&mut rng);
            assert!((1..=6).contains(&die.value()));
        }
    }

    #[test]
    fn test_dice_roll_parse() {
        let roll = DiceRoll::parse("11111", Some(5)).unwrap();
        assert_eq!(roll.as_str(), "11111");
        assert_eq!(roll.len(), 5);

        // Invalid length
        let err = DiceRoll::parse("1111", Some(5)).unwrap_err();
        assert!(matches!(
            err,
            EffDiceError::InvalidRollLength {
                expected: 5,
                found: 4,
                ..
            }
        ));

        // Invalid char
        let err = DiceRoll::parse("11171", Some(5)).unwrap_err();
        assert!(matches!(err, EffDiceError::InvalidDieDigit('7')));
    }

    #[test]
    fn test_dice_roll_sequence_delimited() {
        let seq = DiceRoll::parse_sequence("11111 22222,33333;44444-55555", 5).unwrap();
        assert_eq!(seq.len(), 5);
        assert_eq!(seq[0].as_str(), "11111");
        assert_eq!(seq[1].as_str(), "22222");
        assert_eq!(seq[2].as_str(), "33333");
        assert_eq!(seq[3].as_str(), "44444");
        assert_eq!(seq[4].as_str(), "55555");
    }

    #[test]
    fn test_dice_roll_sequence_continuous() {
        let seq = DiceRoll::parse_sequence("111112222233333", 5).unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq[0].as_str(), "11111");
        assert_eq!(seq[1].as_str(), "22222");
        assert_eq!(seq[2].as_str(), "33333");

        let err = DiceRoll::parse_sequence("111112222", 5).unwrap_err();
        assert!(matches!(err, EffDiceError::InvalidRollInput(_)));
    }

    #[test]
    fn test_dice_roll_sequence_spaced_individual_dice() {
        // 5 single dice
        let seq1 = DiceRoll::parse_sequence("1 1 1 1 1", 5).unwrap();
        assert_eq!(seq1.len(), 1);
        assert_eq!(seq1[0].as_str(), "11111");

        // 10 single dice across two words
        let seq2 = DiceRoll::parse_sequence("1 1 1 1 1  2 2 2 2 2", 5).unwrap();
        assert_eq!(seq2.len(), 2);
        assert_eq!(seq2[0].as_str(), "11111");
        assert_eq!(seq2[1].as_str(), "22222");

        // Comma-separated individual dice
        let seq3 = DiceRoll::parse_sequence("1, 1, 1, 1, 1", 5).unwrap();
        assert_eq!(seq3.len(), 1);
        assert_eq!(seq3[0].as_str(), "11111");

        // 4 single dice when 5 are expected
        let err = DiceRoll::parse_sequence("1 1 1 1", 5).unwrap_err();
        assert!(matches!(
            err,
            EffDiceError::InvalidRollLength {
                expected: 5,
                found: 4,
                ..
            }
        ));

        // Invalid die digit in spaced sequence
        let err = DiceRoll::parse_sequence("1 1 1 7 1", 5).unwrap_err();
        assert!(matches!(err, EffDiceError::InvalidDieDigit('7')));

        // Mismatched token lengths: "11111 2222"
        let err = DiceRoll::parse_sequence("11111 2222", 5).unwrap_err();
        assert!(matches!(
            err,
            EffDiceError::InvalidRollLength { expected: 5, found: 4, roll } if roll == "2222"
        ));
    }

    #[test]
    fn test_dice_roll_random() {
        let mut rng = StdRng::seed_from_u64(12345);
        let roll = DiceRoll::roll(5, &mut rng);
        assert_eq!(roll.len(), 5);
        assert_eq!(roll.as_str().len(), 5);
        for c in roll.as_str().chars() {
            assert!(('1'..='6').contains(&c));
        }
    }
}
