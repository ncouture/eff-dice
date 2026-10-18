use eff_dice::{
    dice::DiceRoll,
    error::EffDiceError,
    passphrase::{PassphraseGenerator, WordTransform},
    wordlist::WordList,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn test_all_local_wordlist_files_load_and_match() {
    // 1. Large list
    let large_tsv = WordList::from_file(repo_path("data/large.txt"), None).unwrap();
    let large_words = WordList::from_file(repo_path("data/large_words.txt"), None).unwrap();
    assert_eq!(large_tsv.len(), 7776);
    assert_eq!(large_words.len(), 7776);
    assert_eq!(large_tsv.dice_per_word(), 5);
    assert_eq!(large_words.dice_per_word(), 5);

    // Verify first and last entries
    assert_eq!(large_tsv.get("11111"), Some("abacus"));
    assert_eq!(large_words.get("11111"), Some("abacus"));
    assert_eq!(large_tsv.get("66666"), Some("zoom"));
    assert_eq!(large_words.get("66666"), Some("zoom"));

    // 2. Small 1 list
    let small_1_tsv = WordList::from_file(repo_path("data/small_1.txt"), None).unwrap();
    let small_1_words = WordList::from_file(repo_path("data/small_1_words.txt"), None).unwrap();
    assert_eq!(small_1_tsv.len(), 1296);
    assert_eq!(small_1_words.len(), 1296);
    assert_eq!(small_1_tsv.dice_per_word(), 4);
    assert_eq!(small_1_words.dice_per_word(), 4);
    assert_eq!(small_1_tsv.get("1111"), Some("acid"));
    assert_eq!(small_1_words.get("1111"), Some("acid"));
    assert_eq!(small_1_tsv.get("6666"), Some("zoom"));
    assert_eq!(small_1_words.get("6666"), Some("zoom"));

    // 3. Small 2 list
    let small_2_tsv = WordList::from_file(repo_path("data/small_2.txt"), None).unwrap();
    let small_2_words = WordList::from_file(repo_path("data/small_2_words.txt"), None).unwrap();
    assert_eq!(small_2_tsv.len(), 1296);
    assert_eq!(small_2_words.len(), 1296);
    assert_eq!(small_2_tsv.dice_per_word(), 4);
    assert_eq!(small_2_words.dice_per_word(), 4);
    assert_eq!(small_2_tsv.get("1111"), Some("aardvark"));
    assert_eq!(small_2_words.get("1111"), Some("aardvark"));
    assert_eq!(small_2_tsv.get("6666"), Some("zucchini"));
    assert_eq!(small_2_words.get("6666"), Some("zucchini"));
}

#[test]
fn test_dice_count_exact_mapping() {
    let wl = WordList::large();
    let mut rng = StdRng::seed_from_u64(42);

    // 5 dice = 1 word
    let p1 =
        PassphraseGenerator::generate_from_dice_count(&wl, 5, " ", WordTransform::None, &mut rng)
            .unwrap();
    assert_eq!(p1.words().len(), 1);
    assert_eq!(p1.total_dice(), 5);

    // 30 dice = 6 words
    let p6 =
        PassphraseGenerator::generate_from_dice_count(&wl, 30, " ", WordTransform::None, &mut rng)
            .unwrap();
    assert_eq!(p6.words().len(), 6);
    assert_eq!(p6.total_dice(), 30);

    // For small list (4 dice/word):
    let wl_small = WordList::small_1();
    // 24 dice = 6 words
    let p_small = PassphraseGenerator::generate_from_dice_count(
        &wl_small,
        24,
        " ",
        WordTransform::None,
        &mut rng,
    )
    .unwrap();
    assert_eq!(p_small.words().len(), 6);
    assert_eq!(p_small.total_dice(), 24);
}

#[test]
fn test_invalid_dice_count_suggestion() {
    let wl = WordList::large(); // 5 dice/word
    let mut rng = StdRng::seed_from_u64(42);

    let err =
        PassphraseGenerator::generate_from_dice_count(&wl, 18, " ", WordTransform::None, &mut rng)
            .unwrap_err();
    match err {
        EffDiceError::InvalidDiceCount {
            dice_count,
            dice_per_word,
            suggested_lower,
            suggested_higher,
        } => {
            assert_eq!(dice_count, 18);
            assert_eq!(dice_per_word, 5);
            assert_eq!(suggested_lower, 15);
            assert_eq!(suggested_higher, 20);
        }
        _ => panic!("Expected InvalidDiceCount"),
    }
}

#[test]
fn test_manual_rolls_passphrase_building() {
    let wl = WordList::large();

    let input = "11111 11112 11113 11114 11115 11116";
    let rolls = DiceRoll::parse_sequence(input, wl.dice_per_word()).unwrap();
    assert_eq!(rolls.len(), 6);

    let pass =
        PassphraseGenerator::from_manual_rolls(&wl, &rolls, "-", WordTransform::None).unwrap();
    assert_eq!(
        pass.text(),
        "abacus-abdomen-abdominal-abide-abiding-ability"
    );
    assert_eq!(pass.total_dice(), 30);
    assert_eq!(pass.words().len(), 6);
}

#[test]
fn test_word_transforms() {
    let wl = WordList::large();
    let rolls = DiceRoll::parse_sequence("11111 66666", 5).unwrap();

    let p_title =
        PassphraseGenerator::from_manual_rolls(&wl, &rolls, ".", WordTransform::TitleCase).unwrap();
    assert_eq!(p_title.text(), "Abacus.Zoom");

    let p_upper =
        PassphraseGenerator::from_manual_rolls(&wl, &rolls, "_", WordTransform::UpperCase).unwrap();
    assert_eq!(p_upper.text(), "ABACUS_ZOOM");

    let p_lower =
        PassphraseGenerator::from_manual_rolls(&wl, &rolls, " ", WordTransform::LowerCase).unwrap();
    assert_eq!(p_lower.text(), "abacus zoom");
}

#[test]
fn test_custom_file_loading() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "11\talpha").unwrap();
    writeln!(file, "12\tbeta").unwrap();
    writeln!(file, "21\tgamma").unwrap();
    writeln!(file, "22\tdelta").unwrap();

    let wl = WordList::from_file(file.path(), Some(2)).unwrap();
    assert_eq!(wl.len(), 4);
    assert_eq!(wl.dice_per_word(), 2);
    assert_eq!(wl.get("11"), Some("alpha"));
    assert_eq!(wl.get("22"), Some("delta"));

    let rolls = DiceRoll::parse_sequence("11 22", 2).unwrap();
    let pass =
        PassphraseGenerator::from_manual_rolls(&wl, &rolls, " ", WordTransform::None).unwrap();
    assert_eq!(pass.text(), "alpha delta");
}

#[test]
fn test_exhaustive_wordlist_file_integrity() {
    let check_pair = |tsv_name: &str, words_name: &str, count: usize, dpw: usize| {
        let tsv = WordList::from_file(repo_path(tsv_name), None).unwrap();
        let words = WordList::from_file(repo_path(words_name), None).unwrap();
        assert_eq!(tsv.len(), count);
        assert_eq!(words.len(), count);
        assert_eq!(tsv.dice_per_word(), dpw);
        assert_eq!(words.dice_per_word(), dpw);

        for i in 0..count {
            let roll = WordList::index_to_roll(i, dpw).unwrap();
            let word_tsv = tsv.get(&roll).unwrap();
            let word_raw = words.get(&roll).unwrap();
            assert_eq!(word_tsv, word_raw, "Mismatch at roll {roll} (index {i})");
        }
    };

    check_pair("data/large.txt", "data/large_words.txt", 7776, 5);
    check_pair("data/small_1.txt", "data/small_1_words.txt", 1296, 4);
    check_pair("data/small_2.txt", "data/small_2_words.txt", 1296, 4);
}

#[test]
fn test_crlf_wordlist_loading() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "11\talpha\r\n12\tbeta\r\n21\tgamma\r\n22\tdelta\r\n").unwrap();

    let wl = WordList::from_file(file.path(), Some(2)).unwrap();
    assert_eq!(wl.len(), 4);
    assert_eq!(wl.get("11"), Some("alpha"));
    assert_eq!(wl.get("22"), Some("delta"));
}

#[test]
fn test_wordlist_comments_and_empty_lines() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# This is a header comment").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "11\tapple").unwrap();
    writeln!(file, "   # Mid-file comment").unwrap();
    writeln!(file, "12\tbanana").unwrap();
    writeln!(file, "\t\t").unwrap();

    let wl = WordList::from_file(file.path(), Some(2)).unwrap();
    assert_eq!(wl.len(), 2);
    assert_eq!(wl.get("11"), Some("apple"));
    assert_eq!(wl.get("12"), Some("banana"));
}

#[test]
fn test_custom_file_phrase_words() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "11\tnew york").unwrap();
    writeln!(file, "12\tsan francisco").unwrap();

    let wl = WordList::from_file(file.path(), Some(2)).unwrap();
    assert_eq!(wl.len(), 2);
    assert_eq!(wl.get("11"), Some("new york"));
    assert_eq!(wl.get("12"), Some("san francisco"));

    let rolls = DiceRoll::parse_sequence("11 12", 2).unwrap();
    let pass =
        PassphraseGenerator::from_manual_rolls(&wl, &rolls, " - ", WordTransform::None).unwrap();
    assert_eq!(pass.text(), "new york - san francisco");
}
