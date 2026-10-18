# EFF Dice

Source: https://www.eff.org/dice

These lists allow you to generate and use passwords based on the rolls of dice.

## Included files

- data/large.txt - EFF's Large Word List
- data/large_words.txt - EFF's Large Word List, with only the words present (i.e. no side numbers)
- data/small_1.txt - EFF's Small Word List 1, for use with 4 dice and featuring only short words
- data/small_1_words.txt - EFF's Small Word List 1, with only the words present (i.e. no side numbers)
- data/small_2.txt - EFF's Small Word List 2, for use with 4 dice and featuring longer words that may be more memorable
- data/small_2_words.txt - EFF's Small Word List 2, with only the words present (i.e. no side numbers)

## Rust Passphrase Generator (`eff-dice`)

This repository includes a Rust implementation and CLI tool (`eff-dice`) to roll dice and generate cryptographically secure passphrases using the EFF wordlists.

### Features

- **Amount of Dice**: Specify the total number of dice to roll per passphrase (`--dice <N>`).
- **Passphrase Count**: Generate as many passphrases as needed (`--count <N>`).
- **Manual Dice Rolls**: Convert physical dice roll results directly into passphrases (`--rolls "<rolls>"` or `--rolls -` from stdin). Supports grouped rolls (`"11111 66666"`), continuous streams (`"1111166666"`), individual spaced dice (`"1 1 1 1 1  6 6 6 6 6"` or `"1, 1, 1, 1, 1"`), and multi-line streams.
- **Wordlist Selection**: Choose between the large list (5 dice per word) and small lists 1 & 2 (4 dice per word) via `--list <large|small1|small2>`, or provide a custom file (`--file <path>`).
- **Formatting & Inspection**: Customize delimiters (`--separator <SEP>`), capitalization (`--capitalize`, `--uppercase`), show rolled dice (`--show-dice`), output dice numbers only (`--dice-only`), or inspect entropy details (`--verbose`, `--entropy`).

### Build Process

Build the release binary using Cargo or Make:

```bash
# Using Cargo
cargo build --release

# Or using Make
make build
```

The executable will be located at `target/release/eff-dice`.

### Testing (TDD Suite)

Run the full suite of unit, integration, and CLI tests:

```bash
# Using Cargo
cargo test --all-targets

# Or using Make
make test
```

### Usage Examples

1. **Roll 25 dice (5 words from Large Wordlist):**
   ```bash
   ./target/release/eff-dice --dice 25
   ```

2. **Roll dice and show the rolled numbers alongside the passphrase:**
   ```bash
   ./target/release/eff-dice --dice 25 --show-dice
   # Output: 14216 35142 52146 62315 24153: cartwheel olive unraveling distinct smolder
   ```

3. **Generate 3 passphrases of 6 words with custom delimiter and capitalization:**
   ```bash
   ./target/release/eff-dice --count 3 --words 6 --separator "-" --capitalize
   ```

4. **Convert manual dice roll results (e.g. from physical dice, spaced or grouped):**
   ```bash
   ./target/release/eff-dice --rolls "11111 66666"
   # Output: abacus zoom

   ./target/release/eff-dice --rolls "1 1 1 1 1  6 6 6 6 6"
   # Output: abacus zoom
   ```

5. **Use EFF Small Wordlist 1 (4 dice per word) with 20 dice and entropy breakdown:**
   ```bash
   ./target/release/eff-dice --list small1 --dice 20 --verbose
   ```

6. **Use a custom wordlist file path:**
   ```bash
   ./target/release/eff-dice --file data/large.txt --rolls "11111 66666"
   # Output: abacus zoom
   ```
