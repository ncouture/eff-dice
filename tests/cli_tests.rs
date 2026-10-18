use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_default_generation() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let words: Vec<&str> = stdout.split_whitespace().collect();
    // Default is 5 words for large list
    assert_eq!(words.len(), 5);
}

#[test]
fn test_cli_specify_words() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-w").arg("7");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let words: Vec<&str> = stdout.split_whitespace().collect();
    assert_eq!(words.len(), 7);
}

#[test]
fn test_cli_specify_dice_count() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-d").arg("25");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let words: Vec<&str> = stdout.split_whitespace().collect();
    // 25 dice / 5 dice-per-word = 5 words
    assert_eq!(words.len(), 5);
}

#[test]
fn test_cli_invalid_dice_count() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-d").arg("22");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid dice count 22"))
        .stderr(predicate::str::contains(
            "Did you mean 20 dice (4 words) or 25 dice (5 words)?",
        ));
}

#[test]
fn test_cli_count_multiple_passphrases() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-c").arg("4");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 4);
}

#[test]
fn test_cli_custom_separator() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-s").arg("---");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("---"));
}

#[test]
fn test_cli_manual_rolls() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111 66666");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("abacus zoom\n"));
}

#[test]
fn test_cli_manual_rolls_stdin() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("-");
    cmd.write_stdin("11111 11112 66666\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("abacus abdomen zoom\n"));
}

#[test]
fn test_cli_small_1_list() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-l").arg("small1").arg("-r").arg("1111 6666");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("acid zoom\n"));
}

#[test]
fn test_cli_small_2_list() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-l").arg("small2").arg("-r").arg("1111 6666");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("aardvark zucchini\n"));
}

#[test]
fn test_cli_casing_options() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111 66666").arg("--capitalize");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("Abacus Zoom\n"));

    let mut cmd_upper = Command::cargo_bin("eff-dice").unwrap();
    cmd_upper.arg("-r").arg("11111 66666").arg("--uppercase");
    cmd_upper
        .assert()
        .success()
        .stdout(predicate::str::diff("ABACUS ZOOM\n"));
}

#[test]
fn test_cli_verbose_mode() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111 66666").arg("-v");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Passphrase:    abacus zoom"))
        .stdout(predicate::str::contains(
            "Dice rolls:    11111 66666 (10 dice)",
        ))
        .stdout(predicate::str::contains("Entropy:"))
        .stdout(predicate::str::contains("1. 11111 -> abacus"))
        .stdout(predicate::str::contains("2. 66666 -> zoom"));
}

#[test]
fn test_cli_entropy_mode() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111 66666").arg("-e");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("abacus zoom"))
        .stdout(predicate::str::contains("bits entropy, 10 dice"));
}

#[test]
fn test_cli_spaced_dice_rolls() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("1 1 1 1 1  6 6 6 6 6");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("abacus zoom\n"));
}

#[test]
fn test_cli_comma_spaced_dice_rolls() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("1, 1, 1, 1, 1");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("abacus\n"));
}

#[test]
fn test_cli_multiline_manual_rolls_stdin() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("-");
    cmd.write_stdin("11111 66666\n22222 33333\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("abacus zoom\ndating handgrip\n"));
}

#[test]
fn test_cli_conflicting_dice_and_words() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-d").arg("20").arg("-w").arg("5");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Conflicting arguments: 20 dice specified, but 5 words require 25 dice (5 per word).",
    ));
}

#[test]
fn test_cli_conflicting_rolls_and_dice() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111").arg("-d").arg("5");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_cli_conflicting_rolls_and_words() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111").arg("-w").arg("1");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

fn repo_path(relative: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn test_cli_conflicting_file_and_list() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-f")
        .arg(repo_path("data/large.txt"))
        .arg("-l")
        .arg("small1");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_cli_custom_file_tsv() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-f")
        .arg(repo_path("data/large.txt"))
        .arg("-r")
        .arg("11111");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("abacus\n"));
}

#[test]
fn test_cli_custom_file_words_only() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-f")
        .arg(repo_path("data/large_words.txt"))
        .arg("-r")
        .arg("11111");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("abacus\n"));
}

#[test]
fn test_cli_custom_file_small_1_tsv() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-f")
        .arg(repo_path("data/small_1.txt"))
        .arg("-r")
        .arg("1111");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("acid\n"));
}

#[test]
fn test_cli_custom_file_small_2_words() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-f")
        .arg(repo_path("data/small_2_words.txt"))
        .arg("-r")
        .arg("1111");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("aardvark\n"));
}

#[test]
fn test_cli_show_dice() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111 66666").arg("--show-dice");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("11111 66666: abacus zoom\n"));
}

#[test]
fn test_cli_dice_only() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-r").arg("11111 66666").arg("--dice-only");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff("11111 66666\n"));
}

#[test]
fn test_cli_invalid_dice_count_below_minimum() {
    let mut cmd = Command::cargo_bin("eff-dice").unwrap();
    cmd.arg("-d").arg("3");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Did you mean 5 dice (1 word)?"));
}
