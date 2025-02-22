use std::vec;

use super::*;
use googletest::prelude::*;

#[googletest::test]
fn test_successful_credential_parsing() {
    let account = "my_account".to_string();
    let data = "my_account - username: user123, password: pass456\nanother_account - username: other, password: secret";

    let credential = Parser::new(data.into());
    let result = credential.get_credentials(None, account.clone());

    expect_that!(
        result,
        ok(eq(&vec![Credential {
            account,
            username: "user123".to_string(),
            password: "pass456".to_string()
        }]))
    )
}

#[googletest::test]
fn test_account_not_found() {
    let account = "unknown_account".to_string();
    let data = "my_account - username: user123, password: pass456";

    let credential = Parser::new(data.to_string());
    let result = credential.get_credentials(None, account);

    expect_that!(
        result,
        err(matches_pattern!(ParsingError::AccountNotFound(_)))
    );
}

#[googletest::test]
fn test_malformed_credentials() {
    let account = "my_account".to_string();
    let data = "my_account - username user123, password pass456".to_string();

    let credential = Parser::new(data);
    let result = credential.get_credentials(None, account.clone());

    expect_that!(
        result,
        ok(eq(&vec![Credential {
            account,
            username: "".into(),
            password: "".into()
        }]))
    )
}

#[googletest::test]
fn test_multiple_accounts() {
    let account = "account".to_string();
    let data = "account - username: user1, password: pass1\n\
                account - username: user2, password: pass2\n\
                account3 - username: user3, password: pass3"
        .to_string();

    let parser = Parser::new(data);
    let result = parser.get_credentials(None, account.clone());

    expect_pred!(result.is_ok()); // Ensure the result is Ok

    let credentials = result.unwrap(); // Extract Vec<Credential>

    expect_that!(
        credentials,
        contains_each![
            eq(&Credential {
                account: account.clone(),
                username: "user1".to_string(),
                password: "pass1".to_string(),
            }),
            eq(&Credential {
                account,
                username: "user2".to_string(),
                password: "pass2".to_string(),
            }),
        ]
    );
}
