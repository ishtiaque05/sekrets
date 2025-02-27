use clap::Parser;
use googletest::prelude::*;
use std::fs::File;
use std::io::Write;
use std::{env, vec};
use tempfile::TempDir;

use crate::cli::confirm_interactive_pass_mode;
use crate::decryptor::decrypt_file;
use crate::{
    cli::{
        generate_strong_password, handle_append, handle_update, print_credentials,
        prompt_user_password, run, Cli, Commands,
    },
    decryptor,
    encryptor::{encrypt_file, ENCRYPTED_FILENAME},
    password_generator::PasswordGenerationError,
    paths::get_encrypted_file_path,
    tests::helpers::create_temp_plaintext_file,
};

fn make_encrypted_file(content: &str) -> String {
    let file_path = create_temp_plaintext_file(content);

    let pass = prompt_user_password();
    encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file")
}

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
            "--account",
            "github",
            "--account",
            "bank"
        ])
        .command,
        eq(&Commands::Decrypt {
            accounts: vec!["github".to_string(), "bank".to_string()],
            usernames: vec![]
        })
    );
}

#[googletest::test]
fn test_cli_decrypt_parsing_with_username() {
    expect_that!(
        Cli::parse_from(vec![
            "sekrets",
            "decrypt",
            "--account",
            "github",
            "--account",
            "bank",
            "--username",
            "foo",
            "-u",
            "bar"
        ])
        .command,
        eq(&Commands::Decrypt {
            accounts: vec!["github".to_string(), "bank".to_string()],
            usernames: vec!["foo".to_string(), "bar".to_string()]
        })
    );
}

#[googletest::test]
fn test_cli_update_creds() {
    expect_that!(
        Cli::parse_from(vec![
            "sekrets",
            "update",
            "--account",
            "github",
            "-u",
            "foo"
        ])
        .command,
        eq(&Commands::Update {
            account: "github".to_string(),
            username: "foo".to_string()
        })
    );
}

#[googletest::test]
fn test_cli_password_generate() {
    expect_that!(
        Cli::parse_from(vec!["sekrets", "generate", "-p"]).command,
        eq(&Commands::Generate {
            generate_flag: true
        })
    );
}

#[googletest::test]
fn test_cli_update_failure() {
    let res = Cli::try_parse_from(vec!["sekrets", "update", "--account", "github"]);

    expect_that!(
        res.unwrap_err().to_string(),
        contains_substring(
            "the following required arguments were not provided:\n  --username <USERNAME>"
        )
    );
}

#[googletest::test]
fn test_decrypt_missing_args() {
    let result = Cli::try_parse_from(vec!["sekrets", "decrypt"]);

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("the following required arguments were not provided:\n  --account")
    );
}

#[googletest::test]
fn test_decrypt_account_not_equal_username() {
    let result = run(Cli::parse_from(vec![
        "sekrets", "decrypt", "-a", "bank", "-u", "foo", "-a", "bar",
    ]));

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("Mismatched accounts and usernames")
    );
}

#[googletest::test]
fn test_decrypt_account_equal_username() {
    env::set_var("TEST_MODE", "1");

    let file_path = create_temp_plaintext_file(
        "github - username: foo, password: bar\nbank - username: abc, password: efg",
    );

    let pass = prompt_user_password();
    let _ = encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Encryption failed");

    let result = run(Cli::parse_from(vec![
        "sekrets", "decrypt", "-a", "bank", "-u", "abc", "-a", "github", "-u", "foo",
    ]));

    expect_pred!(result.is_ok());
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

    expect_pred!(get_encrypted_file_path(ENCRYPTED_FILENAME).exists());

    env::remove_var("TEST_MODE");
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
        "--account",
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
        "--account",
        "bank",
        "--username",
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
        "--account",
        "bank",
        "--username",
        "john_doe",
        "--account",
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
        "--account",
        "bank",
        "--username",
        "john_doe",
    ]));

    expect_pred!(result.is_err());
    expect_that!(
        result.unwrap_err().to_string(),
        contains_substring("does not exist! Encrypt file first")
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_handle_update() {
    env::set_var("TEST_MODE", "1");

    let file_path = create_temp_plaintext_file("github - username: git, password: change_me");
    let pass = prompt_user_password();
    let _ =
        encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

    let _ = handle_update("github".to_string(), "git".to_string());

    let decrypted_data = decryptor::decrypt_file(
        &get_encrypted_file_path(ENCRYPTED_FILENAME).to_string_lossy(),
        &pass,
    )
    .expect("Failed to decrypt file");

    expect_that!(
        decrypted_data,
        contains_substring("github - username: git, password: bar")
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_handle_update_username_not_found() {
    env::set_var("TEST_MODE", "1");

    let file_path = create_temp_plaintext_file("github - username: me, password: change_me");
    let pass = prompt_user_password();
    let _ =
        encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

    let _ = handle_update("github".to_string(), "unknown".to_string());

    let decrypted_data = decryptor::decrypt_file(
        &get_encrypted_file_path(ENCRYPTED_FILENAME).to_string_lossy(),
        &pass,
    )
    .expect("Failed to decrypt file");

    expect_that!(
        decrypted_data,
        contains_substring("github - username: me, password: change_me")
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_handle_update_failure() {
    env::set_var("TEST_MODE", "1");

    let res = handle_update("github".to_string(), "unknown".to_string());

    expect_that!(
        res.unwrap_err().to_string(),
        contains_substring("Failed to read to file: No such file or directory")
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_generate_strong_password() {
    env::set_var("TEST_MODE", "1");

    expect_that!(generate_strong_password(true), ok(()));
    expect_that!(
        generate_strong_password(false)
            .unwrap_err()
            .downcast_ref::<PasswordGenerationError>(),
        some(eq(&PasswordGenerationError::NoChoiceSelected))
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_print_credentials_fail() {
    make_encrypted_file("bank - username: foo, password: bar");

    let res = print_credentials(&["git".to_string()], vec![]);

    expect_that!(res, ok(()))
}

#[googletest::test]
fn test_handle_append_success_nonexisting_acc() {
    env::set_var("TEST_MODE", "1");

    let encrypted_file_path = make_encrypted_file("bank - username: foo, password: bar");
    let _ = handle_append(&["github".to_string()], &["git".to_string()]);

    let data = decrypt_file(&encrypted_file_path, &prompt_user_password()).unwrap();

    expect_that!(
        data,
        contains_substring("github - username: git, password: bar")
    );

    env::remove_var("TEST_MODE");
}

#[googletest::test]
fn test_handle_append_success_existing_acc_pass_update() {
    env::set_var("TEST_PASSWORD_INTERACTIVE", "yes");
    env::set_var("TEST_MODE", "1");

    let encrypted_file_path = make_encrypted_file("bank - username: foo, password: willbechanged");
    let _ = handle_append(&["bank".to_string()], &["foo".to_string()]);

    let data = decrypt_file(&encrypted_file_path, &prompt_user_password()).unwrap();

    expect_that!(
        data,
        contains_substring("bank - username: foo, password: bar")
    );

    env::remove_var("TEST_PASSWORD_INTERACTIVE");
    env::remove_var("TEST_MODE");
}

// #[googletest::test]
// fn test_handle_append_success_existing_acc_no_pass_update() {
//     env::set_var("TEST_PASSWORD_INTERACTIVE", "no");
//     env::set_var("TEST_MODE", "1");

//     let file_path = create_temp_plaintext_file("bank - username: foo, password: willbechanged");

//     let pass = prompt_user_password();
//     encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file");

//     let _= handle_append(&vec!["bank".to_string()], &vec!["foo".to_string()]);

//     let data = decrypt_file(file_path.path().to_str().unwrap(), &pass).unwrap();

//     // expect_that!(data, contains_substring("bank - username: foo, password: willbechanged"));

//     env::remove_var("TEST_PASSWORD_INTERACTIVE");
//     env::remove_var("TEST_MODE");
// }

#[googletest::test]
fn test_confirm_interactive_pass_mode() {
    env::set_var("TEST_PASSWORD_INTERACTIVE", "no");
    expect_that!(confirm_interactive_pass_mode().unwrap(), eq("no"));

    env::set_var("TEST_PASSWORD_INTERACTIVE", "yes");
    expect_that!(confirm_interactive_pass_mode().unwrap(), eq("yes"));

    env::remove_var("TEST_PASSWORD_INTERACTIVE");
}
