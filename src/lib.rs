//! # EFF Dice
//!
//! A Rust implementation of the Electronic Frontier Foundation (EFF) Dice-Generated Passphrases.
//!
//! Based on: <https://www.eff.org/dice>
//!
//! This crate provides tools to roll physical or simulated dice and generate cryptographically
//! secure, human-memorable passphrases using the official EFF wordlists:
//!
//! - **EFF Large Word List** (`data/large.txt`): 7,776 words, 5 dice per word (~12.92 bits of entropy per word).
//! - **EFF Small Word List 1** (`data/small_1.txt`): 1,296 short words, 4 dice per word (~10.34 bits of entropy per word).
//! - **EFF Small Word List 2** (`data/small_2.txt`): 1,296 memorable words, 4 dice per word (~10.34 bits of entropy per word).
//!
//! Additionally, words-only list files (`data/large_words.txt`, `data/small_1_words.txt`, `data/small_2_words.txt`)
//! are located in the `data/` directory and can be loaded dynamically with `WordList::from_file`.

pub mod dice;
pub mod error;
pub mod passphrase;
pub mod wordlist;

pub use dice::{DiceRoll, Die};
pub use error::EffDiceError;
pub use passphrase::{Passphrase, PassphraseGenerator, WordTransform};
pub use wordlist::{WordList, WordListType, LARGE_RAW, SMALL_1_RAW, SMALL_2_RAW};
