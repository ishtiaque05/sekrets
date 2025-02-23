use assert_cmd::Command;
use googletest::expect_pred;
use predicates::prelude::*;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use tempfile::{NamedTempFile, TempDir};

fn create_temp_plaintext_file(content: &str) -> NamedTempFile {
    let tmp_dir = TestCleanup::root_path();
    if !tmp_dir.exists() {
        fs::create_dir_all(&tmp_dir).expect("Failed to create test temp directory");
    }
    let temp_file = NamedTempFile::new_in(tmp_dir).expect("Failed to create temp file");
    let mut file = File::create(temp_file.path()).expect("Failed to open temp file");

    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    file.flush().expect("Failed to flush file");

    temp_file
}

fn run_sekrets_command(args: &[&str]) -> assert_cmd::Command {
    let mut cmd = Command::cargo_bin("sekrets").unwrap();
    cmd.env("TEST_MODE", "1").args(args);

    cmd
}

struct TestCleanup;

impl TestCleanup {
    fn root_path() -> PathBuf {
        let project_root = env::var("CARGO_MANIFEST_DIR").unwrap();

        PathBuf::from(project_root).join("tmp")
    }
    fn clean() {
        let tmp_folder = Self::root_path();

        if tmp_folder.exists() {
            if fs::remove_dir_all(tmp_folder).is_ok() {
                return;
            }
        }
        thread::sleep(Duration::from_millis(100))
    }
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        Self::clean();
    }
}

#[googletest::test]
fn test_full_encrypt_decrypt_flow() {
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

    TestCleanup::clean();
}

#[googletest::test]
fn test_append_flow() {
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

    TestCleanup::clean();
}

#[googletest::test]
fn test_decrypt_with_username_and_account_match() {
    let _cleanup = TestCleanup;

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

    TestCleanup::clean();
}

#[googletest::test]
fn test_decrypt_with_username_and_account_not_match() {
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

    TestCleanup::clean();
}

#[googletest::test]
fn test_decrypt_with_multiple_account_match() {
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

    TestCleanup::clean();
}

#[googletest::test]
fn test_copy() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let plaintext_file = create_temp_plaintext_file(
        "bank - username: user1, password: foo\nbank - username: user2, password: bar",
    );

    let encrypted_path = plaintext_file.path().to_str().unwrap();

    run_sekrets_command(&["encrypt", "-f", encrypted_path])
        .assert()
        .success();

    let destination_dir = temp_dir.path();
    run_sekrets_command(&["copy", "-d", destination_dir.to_str().unwrap()])
        .assert()
        .success();

    let secret_enc_path = destination_dir.join("sekrets.enc");

    expect_pred!(Path::new(&secret_enc_path).exists());

    TestCleanup::clean();
}
