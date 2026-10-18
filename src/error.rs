use std::fmt;

/// Errors that can occur during dice rolling, wordlist parsing, and passphrase generation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EffDiceError {
    /// Dice count specified is not a valid multiple of the wordlist's required dice per word.
    InvalidDiceCount {
        dice_count: usize,
        dice_per_word: usize,
        suggested_lower: usize,
        suggested_higher: usize,
    },
    /// Zero dice specified.
    ZeroDice,
    /// Zero words requested.
    ZeroWords,
    /// Both dice count and word count were provided, but they conflict.
    ConflictingDiceAndWordCount {
        dice_count: usize,
        word_count: usize,
        dice_per_word: usize,
    },
    /// A die character was not in '1'..='6'.
    InvalidDieDigit(char),
    /// Roll sequence length does not match expected length.
    InvalidRollLength {
        expected: usize,
        found: usize,
        roll: String,
    },
    /// The dice roll combination was not found in the wordlist.
    RollNotFound(String),
    /// Empty wordlist provided.
    EmptyWordList,
    /// Wordlist parsing error on a specific line.
    WordListParseError { line_number: usize, details: String },
    /// File I/O error.
    IoError(String),
    /// Invalid roll input format.
    InvalidRollInput(String),
}

impl std::error::Error for EffDiceError {}

impl fmt::Display for EffDiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDiceCount {
                dice_count,
                dice_per_word,
                suggested_lower,
                suggested_higher,
            } => {
                let words_higher = suggested_higher / dice_per_word;
                let higher_word_str = if words_higher == 1 {
                    "1 word".to_string()
                } else {
                    format!("{words_higher} words")
                };

                if *suggested_lower == 0 {
                    write!(
                        f,
                        "Invalid dice count {dice_count}. This wordlist requires {dice_per_word} dice per word. \
                        Did you mean {suggested_higher} dice ({higher_word_str})?"
                    )
                } else {
                    let words_lower = suggested_lower / dice_per_word;
                    let lower_word_str = if words_lower == 1 {
                        "1 word".to_string()
                    } else {
                        format!("{words_lower} words")
                    };
                    write!(
                        f,
                        "Invalid dice count {dice_count}. This wordlist requires {dice_per_word} dice per word. \
                        Did you mean {suggested_lower} dice ({lower_word_str}) or {suggested_higher} dice ({higher_word_str})?"
                    )
                }
            }
            Self::ZeroDice => write!(f, "Dice count must be greater than zero."),
            Self::ZeroWords => write!(f, "Word count must be greater than zero."),
            Self::ConflictingDiceAndWordCount {
                dice_count,
                word_count,
                dice_per_word,
            } => {
                let expected_dice = word_count * dice_per_word;
                let (word_label, verb) = if *word_count == 1 {
                    ("word", "requires")
                } else {
                    ("words", "require")
                };
                write!(
                    f,
                    "Conflicting arguments: {dice_count} dice specified, but {word_count} {word_label} {verb} {expected_dice} dice ({dice_per_word} per word)."
                )
            }
            Self::InvalidDieDigit(c) => {
                write!(
                    f,
                    "Invalid die face '{c}': dice must only have faces 1 through 6."
                )
            }
            Self::InvalidRollLength {
                expected,
                found,
                roll,
            } => {
                write!(
                    f,
                    "Invalid roll length for \"{roll}\": expected {expected} dice digits, but found {found}."
                )
            }
            Self::RollNotFound(roll) => {
                write!(f, "Dice roll \"{roll}\" not found in wordlist.")
            }
            Self::EmptyWordList => write!(f, "Wordlist cannot be empty."),
            Self::WordListParseError {
                line_number,
                details,
            } => {
                write!(f, "Error parsing wordlist at line {line_number}: {details}")
            }
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
            Self::InvalidRollInput(msg) => write!(f, "Invalid roll input: {msg}"),
        }
    }
}

impl From<std::io::Error> for EffDiceError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EffDiceError::InvalidDiceCount {
            dice_count: 23,
            dice_per_word: 5,
            suggested_lower: 20,
            suggested_higher: 25,
        };
        let msg = err.to_string();
        assert!(msg.contains("23"));
        assert!(msg.contains("requires 5 dice"));
        assert!(msg.contains("Did you mean 20 dice (4 words) or 25 dice (5 words)?"));

        let err_low = EffDiceError::InvalidDiceCount {
            dice_count: 3,
            dice_per_word: 5,
            suggested_lower: 0,
            suggested_higher: 5,
        };
        assert_eq!(
            err_low.to_string(),
            "Invalid dice count 3. This wordlist requires 5 dice per word. Did you mean 5 dice (1 word)?"
        );

        let err_conflict = EffDiceError::ConflictingDiceAndWordCount {
            dice_count: 10,
            word_count: 1,
            dice_per_word: 5,
        };
        assert_eq!(
            err_conflict.to_string(),
            "Conflicting arguments: 10 dice specified, but 1 word requires 5 dice (5 per word)."
        );

        let err = EffDiceError::InvalidDieDigit('9');
        assert_eq!(
            err.to_string(),
            "Invalid die face '9': dice must only have faces 1 through 6."
        );

        let err = EffDiceError::ZeroDice;
        assert_eq!(err.to_string(), "Dice count must be greater than zero.");
    }
}
