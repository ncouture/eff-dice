use clap::{Parser, ValueEnum};
use eff_dice::{
    dice::DiceRoll,
    error::EffDiceError,
    passphrase::{PassphraseGenerator, WordTransform},
    wordlist::{WordList, WordListType},
};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

/// Command-line argument parser for EFF Dice Passphrase Generator.
#[derive(Parser, Debug)]
#[command(
    name = "eff-dice",
    author = "EFF Dice Contributors",
    version,
    about = "Generate cryptographically secure passphrases using EFF Dice Wordlists",
    long_about = "A tool to roll dice and generate passphrases based on the Electronic Frontier Foundation (EFF) Dice Wordlists.\n\n\
                 Supports rolling simulated cryptographically secure dice, converting physical dice rolls, and inspecting entropy."
)]
struct Args {
    /// Total number of dice to roll per passphrase (e.g. 25 for 5 words from large list, or 20 for 5 words from small list).
    #[arg(short = 'd', long = "dice", value_name = "COUNT")]
    dice: Option<usize>,

    /// Number of words per passphrase (alternative to --dice; default: 5 for large, 6 for small).
    #[arg(short = 'w', long = "words", value_name = "WORDS")]
    words: Option<usize>,

    /// Number of passphrases to generate.
    #[arg(short = 'c', long = "count", value_name = "NUM")]
    count: Option<usize>,

    /// Built-in EFF wordlist to use (large, small1, small2).
    #[arg(short = 'l', long = "list", value_enum, conflicts_with = "file")]
    list: Option<ListChoice>,

    /// Path to a custom wordlist file (TSV with dice rolls or one word per line).
    #[arg(
        short = 'f',
        long = "file",
        value_name = "PATH",
        conflicts_with = "list"
    )]
    file: Option<PathBuf>,

    /// Expected number of dice per word (auto-detected by default for built-in lists or files).
    #[arg(long = "dice-per-word", value_name = "N")]
    dice_per_word: Option<usize>,

    /// Word separator / delimiter.
    #[arg(
        short = 's',
        long = "separator",
        default_value = " ",
        value_name = "SEP",
        allow_hyphen_values = true
    )]
    separator: String,

    /// Use physical / manual dice roll results (e.g., "11111 23456" or "-" to read from stdin).
    #[arg(
        short = 'r',
        long = "rolls",
        value_name = "ROLLS",
        conflicts_with_all = ["dice", "words", "count"]
    )]
    rolls: Option<String>,

    /// Display dice rolls, breakdown, and entropy for each passphrase.
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Show estimated entropy (in bits) for the passphrase.
    #[arg(short = 'e', long = "entropy")]
    entropy: bool,

    /// Show the rolled dice alongside the passphrase (e.g. "11111 66666: abacus zoom").
    #[arg(long = "show-dice", alias = "show-rolls", alias = "dice-rolls")]
    show_dice: bool,

    /// Output only the rolled dice numbers without passphrase words.
    #[arg(long = "dice-only", alias = "rolls-only")]
    dice_only: bool,

    /// Capitalize the first letter of each word (TitleCase).
    #[arg(long = "capitalize", conflicts_with_all = ["uppercase", "lowercase"])]
    capitalize: bool,

    /// Convert all words to UPPERCASE.
    #[arg(long = "uppercase", conflicts_with_all = ["capitalize", "lowercase"])]
    uppercase: bool,

    /// Convert all words to lowercase.
    #[arg(long = "lowercase", conflicts_with_all = ["capitalize", "uppercase"])]
    lowercase: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum ListChoice {
    /// EFF Large Wordlist (5 dice per word, 7,776 words)
    Large,
    /// EFF Small Wordlist 1 - short words (4 dice per word, 1,296 words)
    #[value(name = "small1", alias = "small-1", alias = "small_1")]
    Small1,
    /// EFF Small Wordlist 2 - memorable words (4 dice per word, 1,296 words)
    #[value(name = "small2", alias = "small-2", alias = "small_2")]
    Small2,
}

impl From<ListChoice> for WordListType {
    fn from(choice: ListChoice) -> Self {
        match choice {
            ListChoice::Large => WordListType::Large,
            ListChoice::Small1 => WordListType::Small1,
            ListChoice::Small2 => WordListType::Small2,
        }
    }
}

fn main() {
    let args = Args::parse();

    if let Err(err) = run(args) {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run(args: Args) -> Result<(), EffDiceError> {
    // 1. Resolve wordlist
    let wordlist = if let Some(path) = &args.file {
        WordList::from_file(path, args.dice_per_word)?
    } else {
        let choice = args.list.unwrap_or(ListChoice::Large);
        WordList::from_type(choice.into())
    };

    // 2. Resolve word casing transform
    let transform = if args.capitalize {
        WordTransform::TitleCase
    } else if args.uppercase {
        WordTransform::UpperCase
    } else if args.lowercase {
        WordTransform::LowerCase
    } else {
        WordTransform::None
    };

    // 3. Check if manual rolls were provided
    if let Some(rolls_input) = args.rolls {
        let raw_rolls = if rolls_input == "-" {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        } else {
            rolls_input
        };

        if raw_rolls.trim().is_empty() {
            return Err(EffDiceError::InvalidRollInput(
                "Input string is empty".to_string(),
            ));
        }

        // Check if there are multiple non-empty lines that can each form independent passphrases
        let lines: Vec<&str> = raw_rolls
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        let mut line_rolls = Vec::new();
        let mut parse_as_lines = lines.len() > 1;
        if parse_as_lines {
            for line in &lines {
                match DiceRoll::parse_sequence(line, wordlist.dice_per_word()) {
                    Ok(r) => line_rolls.push(r),
                    Err(_) => {
                        parse_as_lines = false;
                        break;
                    }
                }
            }
        }

        if parse_as_lines {
            for rolls in line_rolls {
                let passphrase = PassphraseGenerator::from_manual_rolls(
                    &wordlist,
                    &rolls,
                    &args.separator,
                    transform,
                )?;
                print_passphrase(
                    &passphrase,
                    args.verbose,
                    args.entropy,
                    args.show_dice,
                    args.dice_only,
                );
            }
        } else {
            let parsed_rolls = DiceRoll::parse_sequence(&raw_rolls, wordlist.dice_per_word())?;
            let passphrase = PassphraseGenerator::from_manual_rolls(
                &wordlist,
                &parsed_rolls,
                &args.separator,
                transform,
            )?;
            print_passphrase(
                &passphrase,
                args.verbose,
                args.entropy,
                args.show_dice,
                args.dice_only,
            );
        }

        return Ok(());
    }

    // 4. Generate random passphrases using cryptographically secure RNG
    let count = args.count.unwrap_or(1);
    let mut rng = rand::rng();
    let passphrases = PassphraseGenerator::generate_multiple(
        count,
        &wordlist,
        args.dice,
        args.words,
        &args.separator,
        transform,
        &mut rng,
    )?;

    for passphrase in passphrases {
        print_passphrase(
            &passphrase,
            args.verbose,
            args.entropy,
            args.show_dice,
            args.dice_only,
        );
    }

    Ok(())
}

fn print_passphrase(
    passphrase: &eff_dice::Passphrase,
    verbose: bool,
    show_entropy: bool,
    show_dice: bool,
    dice_only: bool,
) {
    let roll_strs: Vec<&str> = passphrase.rolls().iter().map(|r| r.as_str()).collect();
    let rolls_joined = roll_strs.join(" ");

    if dice_only {
        println!("{rolls_joined}");
    } else if verbose {
        println!("{}", passphrase.detailed_summary());
        println!();
    } else if show_dice && show_entropy {
        println!(
            "{rolls_joined}: {}  ({:.2} bits entropy, {} dice)",
            passphrase.text(),
            passphrase.entropy(),
            passphrase.total_dice()
        );
    } else if show_dice {
        println!("{rolls_joined}: {}", passphrase.text());
    } else if show_entropy {
        println!(
            "{}  ({:.2} bits entropy, {} dice)",
            passphrase.text(),
            passphrase.entropy(),
            passphrase.total_dice()
        );
    } else {
        println!("{}", passphrase.text());
    }
}
