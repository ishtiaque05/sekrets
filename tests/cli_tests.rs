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
        .stdout(predicate::str::contains(
            "Enter new password for account: bank, username: user4",
        ))
        .stdout(predicate::str::contains("Password updated successfully"));

    run_sekrets_command(&["generate", "-p"]).assert().success();
    run_sekrets_command(&["find", "-a", "foo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found `foo` matches: 0"))
        .stdout(predicate::str::contains("[]"));

    TestCleanup::clean();
}

#[googletest::test]
fn test_import_creates_version() {
    TestCleanup::clean();

    // Create and encrypt initial file
    let initial = create_temp_plaintext_file("github - username: foo, password: bar");
    run_sekrets_command(&["encrypt", "-f", initial.path().to_str().unwrap()])
        .assert()
        .success();

    // Get path to the encrypted file
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let enc_path = format!("{}/tmp/sekrets_test/sekrets.enc", project_root);
    let import_enc_path = format!("{}/tmp/sekrets_test/import.enc", project_root);

    // Copy the current enc file to use as the import source
    std::fs::copy(&enc_path, &import_enc_path).expect("copy enc to import location to succeed");

    // Re-encrypt with different content so the active file differs from the import file
    let original = create_temp_plaintext_file("original - username: orig, password: origpass");
    run_sekrets_command(&["encrypt", "-f", original.path().to_str().unwrap()])
        .assert()
        .success();

    // Import the previously saved enc file (which contains github/foo)
    run_sekrets_command(&["import", "-f", &import_enc_path])
        .assert()
        .success()
        .stdout(predicates::str::contains("Import successful"));

    // Verify a version was created (the original content should be snapshotted as v1)
    let versions_dir = format!("{}/tmp/sekrets_test/versions", project_root);
    let v1_path = format!("{}/sekrets.v1.enc", versions_dir);
    assert!(
        std::path::Path::new(&v1_path).exists(),
        "Version v1 should exist after import"
    );

    // Verify we can decrypt and see the imported data (github/foo from the copied enc file)
    run_sekrets_command(&["decrypt", "-a", "github"])
        .assert()
        .success()
        .stdout(predicates::str::contains("foo"));

    TestCleanup::clean();
}

#[googletest::test]
fn test_version_list() {
    TestCleanup::clean();

    // Create and encrypt initial file
    let initial = create_temp_plaintext_file("github - username: foo, password: bar");
    run_sekrets_command(&["encrypt", "-f", initial.path().to_str().unwrap()])
        .assert()
        .success();

    // Copy current enc file to use as the import source
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let enc_path = format!("{}/tmp/sekrets_test/sekrets.enc", project_root);
    let import_enc_path = format!("{}/tmp/sekrets_test/import2.enc", project_root);
    std::fs::copy(&enc_path, &import_enc_path).expect("copy enc to import location to succeed");

    // Import to trigger version creation
    run_sekrets_command(&["import", "-f", &import_enc_path])
        .assert()
        .success();

    // List versions — output should mention v1
    run_sekrets_command(&["version", "--list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("v1"));

    TestCleanup::clean();
}

#[googletest::test]
fn test_history_flag() {
    TestCleanup::clean();

    // Create and encrypt initial file
    let initial =
        create_temp_plaintext_file("github - username: histuser, password: initialpass");
    run_sekrets_command(&["encrypt", "-f", initial.path().to_str().unwrap()])
        .assert()
        .success();

    // Update the credential — this pushes the old password into history
    run_sekrets_command(&["update", "-a", "github", "-u", "histuser"])
        .env("PASSWORD_GENERATOR_CHOICE", "4")
        .env("USER_TEST_PASS", "N3wP@ssw0rd!")
        .assert()
        .success()
        .stdout(predicates::str::contains("Password updated successfully"));

    // Decrypt with --history and verify history lines appear in output
    run_sekrets_command(&["decrypt", "-a", "github", "--history"])
        .assert()
        .success()
        .stdout(predicates::str::contains("histuser"))
        .stdout(predicates::str::contains("v1:"));

    TestCleanup::clean();
}
