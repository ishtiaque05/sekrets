use super::*;
use crate::encryption::{decryptor::decrypt_file, encryptor::ENCRYPTED_FILENAME};
use crate::secrets::password_generator::prompt_user_password;
use crate::tests::helpers::make_encrypted_file;
use crate::{helpers::directories::get_encrypted_file_path, secrets::credentials::Credential};
use googletest::prelude::*;
use std::collections::HashMap;
use serde_json;

fn mock_credential(account: &str, username: &str, password: &str) -> Credential {
    Credential::new(account.to_string(), username.to_string(), password.to_string())
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

    expect_that!(data, contains_substring("account1"));
    expect_that!(data, contains_substring("username1"));
    expect_that!(data, contains_substring("password1"));
}

#[googletest::test]
fn test_save_credentials_jsonl_format() {
    let mut manager = CredentialManager {
        master_password: "foo".to_string(),
        credentials: HashMap::new(),
    };

    manager.credentials.insert(
        ("github".to_string(), "user1".to_string()),
        Credential::new("github".to_string(), "user1".to_string(), "pass1".to_string()),
    );

    let result = manager.save_credentials();
    expect_pred!(result.is_ok());

    // Decrypt and verify it's JSONL format
    let data = decrypt_file(
        get_encrypted_file_path(ENCRYPTED_FILENAME).to_str().unwrap(),
        "foo",
    )
    .unwrap();

    // Should start with '{' (JSONL format)
    expect_that!(data.trim().starts_with('{'), eq(true));

    // Should parse back correctly
    let parsed: serde_json::Value = serde_json::from_str(data.trim()).unwrap();
    expect_that!(parsed["account"].as_str().unwrap(), eq("github"));
    expect_that!(parsed["username"].as_str().unwrap(), eq("user1"));
    expect_that!(parsed["password"].as_str().unwrap(), eq("pass1"));
}

#[googletest::test]
fn find_credentials() {
    let manager = CredentialManager {
        master_password: "test".to_string(),
        credentials: HashMap::from([
            (
                ("account1".to_string(), "username1".to_string()),
                mock_credential("account1", "username1", "password1"),
            ),
            (
                ("account2".to_string(), "username2".to_string()),
                mock_credential("account2", "username1", "password2"),
            ),
        ]),
    };

    expect_that!(
        manager.find_all_by_account("account1"),
        eq(&vec!["account1"])
    );
    expect_that!(
        manager.find_all_by_account("acc"),
        unordered_elements_are!["account1", "account2"]
    );

    expect_that!(
        manager.find_all_by_account("bar"),
        unordered_elements_are![]
    );
}

#[googletest::test]
fn test_successful_credential_parsing() {
    let account = "my_account".to_string();

    let _ = make_encrypted_file("my_account - username: user123, password: pass456\nanother_account - username: other, password: secret");
    let credential = CredentialManager::new(prompt_user_password()).expect("not to fail");
    let result = credential.find_any_creds_with(None, account.clone());

    expect_pred!(result.is_ok());
    let creds = result.unwrap();
    expect_pred!(creds.len() == 1);
    expect_that!(creds[0].account, eq("my_account"));
    expect_that!(creds[0].username, eq("user123"));
    expect_that!(creds[0].password, eq("pass456"));
}

#[googletest::test]
fn test_account_not_found() {
    let account = "unknown_account".to_string();

    let _ = make_encrypted_file("my_account - username: user123, password: pass456");
    let credential = CredentialManager::new(prompt_user_password()).expect("not to fail");
    let result = credential.find_any_creds_with(None, account.clone());

    expect_that!(
        result,
        err(matches_pattern!(CredentialError::AccountNotFound(_)))
    );
}

#[googletest::test]
fn test_malformed_credentials() {
    let account = "my_account".to_string();

    let _ = make_encrypted_file("my_account - username user123, password pass456");
    let credential = CredentialManager::new(prompt_user_password()).expect("not to fail");
    let result = credential.find_any_creds_with(None, account.clone());

    expect_that!(
        result,
        err(eq(&CredentialError::AccountNotFound(
            "my_account".to_string()
        )))
    )
}

#[googletest::test]
fn test_multiple_accounts() {
    let account = "account".to_string();
    let data = "account - username: user1, password: pass1\n\
                account - username: user2, password: pass2\n\
                account3 - username: user3, password: pass3";

    let _ = make_encrypted_file(data);
    let credential = CredentialManager::new(prompt_user_password()).expect("not to fail");
    let result = credential.find_any_creds_with(None, account.clone());

    expect_pred!(result.is_ok());

    let credentials = result.unwrap();

    expect_that!(
        credentials,
        contains_each![
            matches_pattern!(Credential {
                account: eq("account"),
                username: eq("user1"),
                password: eq("pass1"),
            }),
            matches_pattern!(Credential {
                account: eq("account"),
                username: eq("user2"),
                password: eq("pass2"),
            }),
        ]
    );
}

#[googletest::test]
fn test_username_account_match() {
    let account = "account".to_string();
    let data = "account - username: user1, password: pass1\n\
                account - username: user2, password: pass2\n\
                account3 - username: user3, password: pass3";

    let _ = make_encrypted_file(data);
    let credential = CredentialManager::new(prompt_user_password()).expect("not to fail");
    let result = credential.find_any_creds_with(Some("user2".to_string()), account.clone());

    expect_pred!(result.is_ok());

    let credentials = result.unwrap();
    expect_pred!(credentials.len() == 1);

    expect_that!(
        credentials[0],
        matches_pattern!(Credential {
            account: eq("account"),
            username: eq("user2"),
            password: eq("pass2"),
        })
    );
}

#[googletest::test]
fn test_username_doesnot_match() {
    let account = "account".to_string();
    let data = "account - username: user1, password: pass1\n\
                account - username: user2, password: pass2\n\
                account3 - username: user3, password: pass3";

    let _ = make_encrypted_file(data);
    let credential = CredentialManager::new(prompt_user_password()).expect("not to fail");
    let result = credential.find_any_creds_with(Some("foo".to_string()), account.clone());

    expect_that!(
        result,
        err(matches_pattern!(
            CredentialError::AccountWithUsernameNotFound(eq("account"), eq("foo"))
        ))
    );
}
