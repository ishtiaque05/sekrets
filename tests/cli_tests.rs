

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::{env, fs, path::Path};


#[googletest::test]
fn test_full_encrypt_decrypt_flow() {
    env::set_var("TEST_MODE", "1");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let plaintext_file = temp_dir.path().join("sekrets.txt");
    fs::write(&plaintext_file, "github - username: foo, password: bar")
        .expect("Failed to write temp file");


    let mut cmd_encrypt = Command::cargo_bin("sekrets").unwrap();
    cmd_encrypt.args(&["encrypt", "-f", plaintext_file.to_str().unwrap()]);

    cmd_encrypt.assert()
        .success()
        .stdout(predicate::str::contains("Encrypting file:"))
        .stdout(predicate::str::contains("Encrypted file created:"));

    let mut cmd_decrypt = Command::cargo_bin("sekrets").unwrap();
    cmd_decrypt.args(&["decrypt", "-a", "github"]);

    cmd_decrypt.assert()
        .success()
        .stdout(predicate::str::contains("Account: github - Username:"))
        .stdout(predicate::str::contains("Password:"));

    let test_temp_dir = Path::new("./tmp/sekrets_test");
    if test_temp_dir.exists() {
        fs::remove_dir_all(test_temp_dir).expect("Failed to delete test temp directory");
    }

    env::remove_var("TEST_MODE");
}