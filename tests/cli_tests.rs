

use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use tempfile::NamedTempFile;
use std::io::Write;


#[googletest::test]
fn test_full_encrypt_decrypt_flow() {
    // Set the environment variable so that prompt_user_password returns "foo"
    env::set_var("TEST_MODE", "1");

    let mut temp_file = NamedTempFile::new().expect("to be successful");
    writeln!(temp_file, "github - username: foo, password: bar").expect("to be written");
    let file_path = temp_file.path().to_str().expect("to be created");

    let mut cmd_encrypt = Command::cargo_bin("sekrets").unwrap();
    cmd_encrypt.args(&["encrypt", "-f", file_path]);

    cmd_encrypt.assert()
        .success()
        .stdout(predicate::str::contains("Encrypting file:"))
        .stdout(predicate::str::contains("Encrypted file created:"));

    let mut cmd_decrypt = Command::cargo_bin("sekrets").unwrap();

    cmd_decrypt.args(&["decrypt", "-f", &format!("{}.enc", file_path), "-a", "github"]);
    cmd_decrypt.assert()
        .success()
        .stdout(predicate::str::contains("Decrypting file:"))
        .stdout(predicate::str::contains("Account: github - Username:"))
        .stdout(predicate::str::contains("Password:"));

    env::remove_var("TEST_MODE");
}