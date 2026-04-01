use assert_cmd::Command;
use googletest::expect_pred;
use predicates::prelude::*;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

/// Each test gets its own isolated temp directory via SEKRETS_TEST_DIR.
/// This allows integration tests to run in parallel without races.
struct TestEnv {
    dir: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        Self {
            dir: TempDir::new().expect("Failed to create test temp directory"),
        }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn path_str(&self) -> String {
        self.dir.path().to_string_lossy().to_string()
    }

    fn create_plaintext_file(&self, content: &str) -> PathBuf {
        let file_path = self.dir.path().join("plaintext.txt");
        let mut file = File::create(&file_path).expect("Failed to create temp file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to temp file");
        file.flush().expect("Failed to flush file");
        file_path
    }

    fn run(&self, args: &[&str]) -> assert_cmd::Command {
        let mut cmd = Command::cargo_bin("sekrets").unwrap();
        cmd.env("SEKRETS_TEST_DIR", self.path_str())
            .env("TEST_MODE", "1")
            .args(args);
        cmd
    }
}

#[googletest::test]
fn test_sekrets_tool() {
    let env = TestEnv::new();

    let plaintext_file = env.create_plaintext_file(
        "github - username: foo, password: bar\ngithub - username: user1, password: pass1\nbank - username: user3, password: bankpass",
    );

    env.run(&["encrypt", "-f", plaintext_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Encrypting file:"))
        .stdout(predicate::str::contains("Encrypted file created:"));

    env.run(&["decrypt", "-a", "github"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Account: github - username: foo, Password: bar").not())
        .stdout(predicate::str::contains(
            "Account: github - Username: user1, Password: pass1",
        ));

    env.run(&["append", "-a", "bank", "-u", "user4"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Adding new credential for account: bank, username: user4",
        ));

    env.run(&["decrypt", "-a", "bank", "-u", "user4"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Account: bank - Username: user4, Password: bar",
        ));

    env.run(&[
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

    let copy_dest = env.path().join("copy_output");
    fs::create_dir_all(&copy_dest).unwrap();
    env.run(&["copy", "-d", copy_dest.to_str().unwrap()])
        .assert()
        .success();

    let secret_enc_path = copy_dest.join("sekrets.enc");
    expect_pred!(Path::new(&secret_enc_path).exists());

    env.run(&["update", "-a", "bank", "-u", "user4"])
        .env("PASSWORD_GENERATOR_CHOICE", "4")
        .env("USER_TEST_PASS", "A^u4IfqU#PRla8+e")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Enter new password for account: bank, username: user4",
        ))
        .stdout(predicate::str::contains("Password updated successfully"));

    env.run(&["generate", "-p"]).assert().success();
    env.run(&["find", "-a", "foo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found `foo` matches: 0"))
        .stdout(predicate::str::contains("[]"));
}

#[googletest::test]
fn test_import_creates_version() {
    let env = TestEnv::new();

    // Create and encrypt initial file
    let initial = env.create_plaintext_file("github - username: foo, password: bar");
    env.run(&["encrypt", "-f", initial.to_str().unwrap()])
        .assert()
        .success();

    // Copy the current enc file to use as the import source
    let enc_path = env.path().join("sekrets.enc");
    let import_enc_path = env.path().join("import.enc");
    fs::copy(&enc_path, &import_enc_path).expect("copy enc to import location to succeed");

    // Re-encrypt with different content so the active file differs from the import file
    let original = env.create_plaintext_file("original - username: orig, password: origpass");
    env.run(&["encrypt", "-f", original.to_str().unwrap()])
        .assert()
        .success();

    // Import the previously saved enc file (which contains github/foo)
    env.run(&["import", "-f", import_enc_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("Import successful"));

    // Verify a version was created (the original content should be snapshotted as v1)
    let v1_path = env.path().join("versions").join("sekrets.v1.enc");
    assert!(v1_path.exists(), "Version v1 should exist after import");

    // Verify we can decrypt and see the imported data (github/foo from the copied enc file)
    env.run(&["decrypt", "-a", "github"])
        .assert()
        .success()
        .stdout(predicates::str::contains("foo"));
}

#[googletest::test]
fn test_version_list() {
    let env = TestEnv::new();

    // Create and encrypt initial file
    let initial = env.create_plaintext_file("github - username: foo, password: bar");
    env.run(&["encrypt", "-f", initial.to_str().unwrap()])
        .assert()
        .success();

    // Copy current enc file to use as the import source
    let enc_path = env.path().join("sekrets.enc");
    let import_enc_path = env.path().join("import2.enc");
    fs::copy(&enc_path, &import_enc_path).expect("copy enc to import location to succeed");

    // Import to trigger version creation
    env.run(&["import", "-f", import_enc_path.to_str().unwrap()])
        .assert()
        .success();

    // List versions — output should mention v1
    env.run(&["version", "--list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("v1"));
}

#[googletest::test]
fn test_history_flag() {
    let env = TestEnv::new();

    // Create and encrypt initial file
    let initial = env.create_plaintext_file("github - username: histuser, password: initialpass");
    env.run(&["encrypt", "-f", initial.to_str().unwrap()])
        .assert()
        .success();

    // Update the credential — this pushes the old password into history
    env.run(&["update", "-a", "github", "-u", "histuser"])
        .env("PASSWORD_GENERATOR_CHOICE", "4")
        .env("USER_TEST_PASS", "N3wP@ssw0rd!")
        .assert()
        .success()
        .stdout(predicates::str::contains("Password updated successfully"));

    // Decrypt with --history and verify history lines appear in output
    env.run(&["decrypt", "-a", "github", "--history"])
        .assert()
        .success()
        .stdout(predicates::str::contains("histuser"))
        .stdout(predicates::str::contains("v1:"));
}
