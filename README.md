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

## How to Generate a Passphrase (EFF Directions)

This method uses physical dice rolls and EFF's wordlists to create memorable, brute-force resistant passphrases. For most applications (such as password manager master passwords and full-disk encryption), we recommend generating a **six-word passphrase** using EFF's Long Wordlist.

### Step-by-Step Instructions

1. **Step 1: Roll five dice all at once.** Note the faces that come up from left to right without looking at the wordlist yet. *(On custom EFF dice, the EFF logo is equivalent to rolling a 1).*
2. **Step 2: Record the numbers.** For example, rolling `4, 3, 4, 6, 3` gives `43463`.
3. **Step 3: Look up the word.** Open EFF's Long Wordlist (`data/large.txt`) and find the number matching `43463`.
4. **Step 4: Record the word.** Matching `43463` yields the word `"panoramic"`. Write it down as the first word of your passphrase.
5. **Step 5: Repeat 5 more times.** Roll five dice and record the word five more times to get a total of **six words**.
   - Example resulting passphrase:
     ```text
     panoramic nectar precut smith banana handclap
     ```
6. **Step 6: Create a mnemonic.** Create a story, scenario, or sentence to remember your phrase in order:
   > *"The **panoramic** view, as I tasted the **nectar** of a **precut** granny **smith** apple and **banana**, deserved a **handclap**."*

### Security and Entropy

- **Entropy per Word (Long List)**: Each 5-dice roll has $6^5 = 7{,}776$ possible outcomes, providing $\log_2(7776) \approx 12.92\text{ bits}$ of entropy per word.
- **Passphrase Strength (6 words)**: A six-word passphrase from the long list has $7776^6 = 221{,}073{,}919{,}720{,}733{,}357{,}899{,}776$ (or $\approx 2^{77.55}$) alternatives, providing over **77.5 bits of entropy**. This makes it virtually impossible to brute force with modern hardware.

### Wordlists Overview

| Wordlist | Path | Dice per Word | Words | Entropy / Word | Recommended Length |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **Long Wordlist** | `data/large.txt` | 5 | 7,776 | ~12.92 bits | $\ge$ 6 words (~77.5 bits) |
| **Short Wordlist #1** (short words) | `data/small_1.txt` | 4 | 1,296 | ~10.34 bits | $\ge$ 7–8 words (~72–82 bits) |
| **Short Wordlist #2** (memorable words) | `data/small_2.txt` | 4 | 1,296 | ~10.34 bits | $\ge$ 7–8 words (~72–82 bits) |

When using either of the short wordlists, roll **four dice** per word instead of five. Because short lists have 1,296 words ($6^4$), each word contributes $\approx 10.34\text{ bits}$, so using 7 to 8 words is recommended to reach equivalent strength.

---

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

1. **Roll 30 dice (6 words from Large Wordlist):**
   ```bash
   ./target/release/eff-dice --words 6
   ```

2. **Roll dice and show the rolled numbers alongside the passphrase:**
   ```bash
   ./target/release/eff-dice --words 6 --show-dice
   # Output: 43331 41326 56222 52146 41531 44134: overthrow moonlight starship revivable mundane payback
   ```

3. **Generate 3 passphrases of 6 words with custom delimiter and capitalization:**
   ```bash
   ./target/release/eff-dice --count 3 --words 6 --separator "-" --capitalize
   ```

4. **Convert manual dice roll results (e.g. from physical dice, spaced or grouped):**
   ```bash
   ./target/release/eff-dice --rolls "43463 36154 44535 55421 14413 31316"
   # Output: panoramic nectar precut smith banana handclap

   ./target/release/eff-dice --rolls "1 1 1 1 1  6 6 6 6 6"
   # Output: abacus zoom
   ```

5. **Show full entropy and roll breakdown:**
   ```bash
   ./target/release/eff-dice --words 6 --verbose
   ```

6. **Use EFF Small Wordlist 1 (4 dice per word) with 28 dice (7 words):**
   ```bash
   ./target/release/eff-dice --list small1 --dice 28 --verbose
   ```

7. **Use a custom wordlist file path:**
   ```bash
   ./target/release/eff-dice --file data/large.txt --rolls "11111 66666"
   # Output: abacus zoom
   ```
