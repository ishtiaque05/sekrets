use assert_cmd::Command;
use googletest::expect_pred;
use predicates::prelude::*;
use std::{env, fs::File, io::Write, path::Path};
use tempfile::{NamedTempFile, TempDir};

fn create_temp_plaintext_file(content: &str) -> NamedTempFile {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut file = File::create(temp_file.path()).expect("Failed to open temp file");

    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    file.flush().expect("Failed to flush file");

    temp_file
}

fn run_sekrets_command(args: &[&str]) -> assert_cmd::Command {
    let mut cmd = Command::cargo_bin("sekrets").unwrap();
    cmd.args(args);
    cmd
}

#[googletest::test]
fn test_full_encrypt_decrypt_flow() {
    env::set_var("TEST_MODE", "1");

    let plaintext_file = create_temp_plaintext_file("github - username: foo, password: bar");

    run_sekrets_command(&["encrypt", "-f", plaintext_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Encrypting file:"))
        .stdout(predicate::str::contains("Encrypted file created:"));

    run_sekrets_command(&["decrypt", "-a", "github"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Account: github - Username:"))
        .stdout(predicate::str::contains("Password:"));

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_append_flow() {
    env::set_var("TEST_MODE", "1");

    let plaintext_file = create_temp_plaintext_file("bank - username: foo, password: bar");
    run_sekrets_command(&["encrypt", "-f", plaintext_file.path().to_str().unwrap()])
        .assert()
        .success();

    run_sekrets_command(&["append", "-a", "bank", "-u", "user2"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Enter password credential for account: bank, username: user2",
        ));

    run_sekrets_command(&["decrypt", "-a", "bank"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Account: bank - Username: foo, Password: bar",
        ))
        .stdout(predicate::str::contains(
            "Account: bank - Username: user2, Password: foo",
        ));

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_decrypt_with_username_and_account_match() {
    env::set_var("TEST_MODE", "1");

    let plaintext_file = create_temp_plaintext_file(
        "bank - username: user1, password: foo\nbank - username: user2, password: bar",
    );
    run_sekrets_command(&["encrypt", "-f", plaintext_file.path().to_str().unwrap()])
        .assert()
        .success();

    run_sekrets_command(&["decrypt", "-a", "bank", "-u", "user2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Account: bank - Username: user1, Password: foo").not())
        .stdout(predicate::str::contains(
            "Account: bank - Username: user2, Password: bar",
        ));

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_decrypt_with_username_and_account_not_match() {
    env::set_var("TEST_MODE", "1");

    let plaintext_file = create_temp_plaintext_file(
        "bank - username: user1, password: foo\nbank - username: user2, password: bar",
    );
    run_sekrets_command(&["encrypt", "-f", plaintext_file.path().to_str().unwrap()])
        .assert()
        .success();

    run_sekrets_command(&[
        "decrypt",
        "-a",
        "bank",
        "-u",
        "user3",
        "--account",
        "bank",
        "--username",
        "user2",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "No credentials found for account: `bank\' with username: `user3\'",
    ))
    .stdout(predicate::str::contains(
        "Account: bank - Username: user2, Password: bar",
    ));

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_decrypt_with_multiple_account_match() {
    env::set_var("TEST_MODE", "1");

    let plaintext_file = create_temp_plaintext_file(
        "bank - username: user1, password: foo\nbank - username: user2, password: bar",
    );
    run_sekrets_command(&["encrypt", "-f", plaintext_file.path().to_str().unwrap()])
        .assert()
        .success();

    run_sekrets_command(&["decrypt", "-a", "bank"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Account: bank - Username: user1, Password: foo",
        ))
        .stdout(predicate::str::contains(
            "Account: bank - Username: user2, Password: bar",
        ));

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_copy() {
    env::set_var("TEST_MODE", "1");

    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let plaintext_file = create_temp_plaintext_file(
        "bank - username: user1, password: foo\nbank - username: user2, password: bar",
    );

    run_sekrets_command(&["encrypt", "-f", plaintext_file.path().to_str().unwrap()])
        .assert()
        .success();

    run_sekrets_command(&["copy", "-d", temp_dir.path().to_str().unwrap()])
        .assert()
        .success();

    let secret_enc_path = temp_dir.path().join("sekrets.enc");

    expect_pred!(Path::new(&secret_enc_path).exists());
}
