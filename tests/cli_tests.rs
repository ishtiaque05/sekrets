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
use tempfile::NamedTempFile;

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
            fs::remove_dir_all(tmp_folder).expect("to be removed");
            thread::sleep(Duration::from_millis(100))
        }
    }
}

#[googletest::test]
fn test_sekrets_tool() {
    let plaintext_file = create_temp_plaintext_file(
        "github - username: foo, password: bar\ngithub - username: user1, password: pass1\nbank - username: user3, password: bankpass");

    run_sekrets_command(&["encrypt", "-f", plaintext_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Encrypting file:"))
        .stdout(predicate::str::contains("Encrypted file created:"));

    run_sekrets_command(&["decrypt", "-a", "github"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Account: github - username: foo, Password: bar").not())
        .stdout(predicate::str::contains(
            "Account: github - Username: user1, Password: pass1",
        ));

    run_sekrets_command(&["append", "-a", "bank", "-u", "user4"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Adding new credential for account: bank, username: user4",
        ));

    run_sekrets_command(&["decrypt", "-a", "bank", "-u", "user4"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Account: bank - Username: user4, Password: bar",
        ));

    run_sekrets_command(&[
        "decrypt",
        "-a",
        "bank",
        "-u",
        "userx",
        "--account",
        "bank",
        "--username",
        "user4",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "No credentials found for account: `bank\' with username: `userx\'",
    ))
    .stdout(predicate::str::contains(
        "Account: bank - Username: user4, Password: bar",
    ));

    run_sekrets_command(&["copy", "-d", TestCleanup::root_path().to_str().unwrap()])
        .assert()
        .success();

    let secret_enc_path = TestCleanup::root_path().join("sekrets.enc");

    expect_pred!(Path::new(&secret_enc_path).exists());

    run_sekrets_command(&["update", "-a", "bank", "-u", "user4"])
        .env("PASSWORD_GENERATOR_CHOICE", "4")
        .env("USER_TEST_PASS", "A^u4IfqU#PRla8+e")
        .assert()
        .success()
        .stdout(predicate::str::contains("Enter new password for account: bank, username: user4"))
        .stdout(predicate::str::contains("Password updated successfully"));

    run_sekrets_command(&["generate", "-p"]).assert().success();

    TestCleanup::clean();
}
