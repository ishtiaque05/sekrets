use clap::Parser;
use googletest::prelude::*;
use std::env;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

use crate::{
    cli::{prompt_user_password, run, Cli, Commands},
    decryptor,
    encryptor::{encrypt_file, ENCRYPTED_FILENAME},
    paths::get_encrypted_file_path,
    tests::helpers::create_temp_plaintext_file,
};

#[googletest::test]
fn test_cli_encrypt_parsing() {
    expect_that!(
        Cli::parse_from(vec!["sekrets", "encrypt", "--file", "../fixtures/foo.txt"]).command,
        eq(&Commands::Encrypt {
            file: "../fixtures/foo.txt".to_string()
        })
    );
}

#[googletest::test]
fn test_missing_file_encrypt() {
    let result = Cli::try_parse_from(vec!["sekrets", "encrypt"]);

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("the following required arguments were not provided:\n  --file <FILE>")
    );
}

#[googletest::test]
fn test_cli_decrypt_parsing() {
    expect_that!(
        Cli::parse_from(vec![
            "sekrets",
            "decrypt",
            "--accounts",
            "github",
            "--accounts",
            "bank"
        ])
        .command,
        eq(&Commands::Decrypt {
            accounts: vec!["github".to_string(), "bank".to_string()]
        })
    );
}

#[googletest::test]
fn test_decrypt_missing_args() {
    let result = Cli::try_parse_from(vec!["sekrets", "decrypt"]);

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("the following required arguments were not provided:\n  --accounts")
    );
}

#[googletest::test]
fn test_run_encrypt_command() {
    env::set_var("TEST_MODE", "1");

    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let file_path = temp_dir.path().join("test_input.txt");

    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "github - username: foo, password: bar").expect("Failed to write to file");

    expect_pred!(run(Cli::parse_from(vec![
        "sekrets",
        "encrypt",
        "--file",
        file_path.to_str().unwrap()
    ]))
    .is_ok());
}

#[googletest::test]
fn test_run_decrypt_command() {
    env::set_var("TEST_MODE", "1");

    let file_path = create_temp_plaintext_file("github - username: foo, password: bar");

    let pass = prompt_user_password();
    let _ = encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Encryption failed");

    expect_pred!(run(Cli::parse_from(vec![
        "sekrets",
        "decrypt",
        "--accounts",
        "github"
    ]))
    .is_ok());

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_prompt_user_password_mocked() {
    env::set_var("TEST_MODE", "1");
    expect_that!(prompt_user_password(), eq("foo"));

    env::remove_var("TEST_MODE")
}

#[googletest::test]
fn test_run_copy_command() {
    env::set_var("TEST_MODE", "1");

    let dest_dir = TempDir::new().expect("Failed to create destination directory");
    let file_path = create_temp_plaintext_file("hello rust");

    let pass = prompt_user_password();
    let _ =
        encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

    expect_pred!(run(Cli::parse_from(vec![
        "sekrets",
        "copy",
        "--dest",
        dest_dir.path().to_str().unwrap(),
    ]))
    .is_ok());

    expect_pred!(dest_dir.path().join(ENCRYPTED_FILENAME).exists());

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_run_append_command() {
    env::set_var("TEST_MODE", "1");

    let file_path = create_temp_plaintext_file("github - username: foo, password: bar");
    let pass = prompt_user_password();
    let _ =
        encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

    expect_pred!(run(Cli::parse_from(vec![
        "sekrets",
        "append",
        "--accounts",
        "bank",
        "--usernames",
        "john_doe",
    ]))
    .is_ok());

    let decrypted_data = decryptor::decrypt_file(
        &get_encrypted_file_path(ENCRYPTED_FILENAME).to_string_lossy(),
        &pass,
    )
    .expect("Failed to decrypt file");

    expect_that!(
        decrypted_data,
        contains_substring("bank - username: john_doe")
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_run_append_mismatched_accounts_usernames() {
    env::set_var("TEST_MODE", "1");

    let file_path = create_temp_plaintext_file("github - username: foo, password: bar");
    let pass = prompt_user_password();
    let _ =
        encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

    let result = run(Cli::parse_from(vec![
        "sekrets",
        "append",
        "--accounts",
        "bank",
        "--usernames",
        "john_doe",
        "--accounts",
        "email",
    ]));

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("Mismatched accounts and usernames")
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_run_append_no_encrypted_file() {
    env::set_var("TEST_MODE", "1");

    let result = run(Cli::parse_from(vec![
        "sekrets",
        "append",
        "--accounts",
        "bank",
        "--usernames",
        "john_doe",
    ]));

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("does not exist! Encrypt file first")
    );

    env::remove_var("TEST_MODE");
}
