use super::*;
use crate::encryption::{decryptor::decrypt_file, encryptor::ENCRYPTED_FILENAME};
use crate::{helpers::directories::get_encrypted_file_path, secrets::credentials::Credential};
use googletest::prelude::*;
use std::collections::HashMap;

fn mock_credential(account: &str, username: &str, password: &str) -> Credential {
    Credential {
        account: account.to_string(),
        username: username.to_string(),
        password: password.to_string(),
    }
}

#[googletest::test]
fn test_find_creds_success() {
    let mut manager = CredentialManager {
        master_password: "test".to_string(),
        credentials: HashMap::new(),
    };

    manager.credentials.insert(
        ("account1".to_string(), "username1".to_string()),
        mock_credential("account1", "username1", "password1"),
    );

    let cred = manager.find_creds("account1", "username1");

    expect_that!(cred.is_some(), eq(true));
    expect_that!(cred.unwrap().password, eq(&"password1".to_string()));
}

#[googletest::test]
fn test_find_creds_not_found() {
    let mut manager = CredentialManager {
        master_password: "test".to_string(),
        credentials: HashMap::new(),
    };

    let cred = manager.find_creds("nonexistent_account", "nonexistent_user");

    expect_that!(cred.is_none(), eq(true));
}

#[googletest::test]
fn test_save_credentials() {
    let mut manager = CredentialManager {
        master_password: "foo".to_string(),
        credentials: HashMap::new(),
    };

    manager.credentials.insert(
        ("account1".to_string(), "username1".to_string()),
        mock_credential("account1", "username1", "password1"),
    );

    let result = manager.save_credentials();

    expect_pred!(result.is_ok());
    let data = decrypt_file(
        get_encrypted_file_path(ENCRYPTED_FILENAME)
            .to_str()
            .unwrap(),
        "foo",
    )
    .unwrap();

    expect_that!(
        data,
        contains_substring("account1 - username: username1, password: password1")
    )
}
