use googletest::prelude::*;
use std::env;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

use crate::{
    cli::{build_cli, prompt_user_password, run},
    decryptor,
    encryptor::{encrypt_file, ENCRYPTED_FILENAME},
    paths::get_encrypted_file_path,
    tests::helpers::create_temp_plaintext_file,
};

#[googletest::test]
fn test_cli_encrypt_parsing() {
    let cli = build_cli();
    let matches = cli.get_matches_from(vec!["sekret", "encrypt", "-f", "../fixtures/foo.txt"]);

    let (subcommand_name, sub_matches) = matches.subcommand().expect("Expected a subcommand");
    expect_that!(subcommand_name, eq("encrypt"));

    let file = sub_matches
        .get_one::<String>("file")
        .expect("File not found");
    expect_that!(file, eq("../fixtures/foo.txt"));
}

#[googletest::test]
fn test_missing_file_encrypt() {
    let cli = build_cli();

    let result = cli.try_get_matches_from(vec!["sekrets", "encrypt"]);

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("the following required arguments were not provided:\n  --file <FILE>")
    );
}

#[googletest::test]
fn test_cli_decrypt_parsing() {
    let cli = build_cli();
    let matches = cli.get_matches_from(vec!["sekrets", "decrypt", "-a", "github", "-a", "bank"]);

    let (subcommand_name, sub_matches) = matches.subcommand().expect("Expected a subcommand");
    expect_that!(subcommand_name, eq("decrypt"));

    let accounts: Vec<_> = sub_matches
        .get_many::<String>("accounts")
        .expect("Accounts not found")
        .collect();

    expect_that!(accounts.len(), eq(2));
    expect_that!(accounts[0], eq("github"));
    expect_that!(accounts[1], eq("bank"));
}

#[googletest::test]
fn test_decrypt_missing_args() {
    let cli = build_cli();
    let result = cli.try_get_matches_from(vec!["sekrets", "decrypt"]);

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

    let file_path_str = file_path
        .to_str()
        .expect("Failed to convert file path to string");

    let matches = build_cli()
        .try_get_matches_from(vec!["sekrets", "encrypt", "--file", file_path_str])
        .expect("Failed to parse arguments");

    let result = run(&matches);

    expect_pred!(result.is_ok());
}

#[googletest::test]
fn test_run_decrypt_command() {
    env::set_var("TEST_MODE", "1");

    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    let file_path = temp_dir.path().join("test_input.txt");

    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "github - username: foo, password: bar").expect("Failed to write to file");

    let file_path_str = file_path
        .to_str()
        .expect("Failed to convert file path to string");

    let pass = prompt_user_password();
    let _ = encrypt_file(file_path_str, &pass).expect("not to fail");

    let matches = build_cli()
        .try_get_matches_from(vec!["sekrets", "decrypt", "--accounts", "github"])
        .expect("Failed to parse arguments");

    let result = run(&matches);

    expect_pred!(result.is_ok());

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_prompt_user_password_mocked() {
    std::env::set_var("TEST_MODE", "1"); // Mock input
    let password = prompt_user_password();
    expect_that!(password, eq(&"foo".to_string()));
}

#[googletest::test]
fn test_run_copy_command() {
    env::set_var("TEST_MODE", "1");

    let dest_dir = TempDir::new().expect("Failed to create destination directory");

    let file_path = create_temp_plaintext_file("hello rust");

    let pass = prompt_user_password();
    let _ =
        encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

    let matches = build_cli()
        .try_get_matches_from(vec![
            "sekrets",
            "copy",
            "--dest",
            dest_dir.path().to_str().expect("Failed to convert path"),
        ])
        .expect("Failed to parse arguments");

    let result = run(&matches);

    expect_pred!(result.is_ok());

    let copied_file = dest_dir.path().join(ENCRYPTED_FILENAME);
    expect_pred!(copied_file.exists());

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_run_append_command() {
    env::set_var("TEST_MODE", "1");

    let file_path = create_temp_plaintext_file("github - username: foo, password: bar");

    let pass = prompt_user_password();
    let _ =
        encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

    let matches = build_cli()
        .try_get_matches_from(vec![
            "sekrets",
            "append",
            "--account",
            "bank",
            "--username",
            "john_doe",
            "--password",
            "mysecurepass",
        ])
        .expect("Failed to parse arguments");

    let result = run(&matches);
    expect_pred!(result.is_ok());

    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);
    let decrypted_data = decryptor::decrypt_file(&encrypted_filepath.to_string_lossy(), &pass)
        .expect("Failed to decrypt file");

    expect_that!(
        decrypted_data,
        contains_substring("bank - username: john_doe, password: mysecurepass")
    );

    env::remove_var("TEST_MODE");
}
