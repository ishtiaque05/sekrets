use super::*;
use googletest::prelude::*;

#[googletest::test]
fn test_successful_credential_parsing() {
    let account = "my_account".to_string();
    let data = "my_account - username: user123, password: pass456\nanother_account - username: other, password: secret";

    let credential = Parser::new(account.clone());
    let result = credential.get_credentials(data.to_string());

    expect_that!(
        result,
        ok(eq(&Credential {
            account: account,
            username: "user123".to_string(),
            password: "pass456".to_string()
        }))
    )
}

#[googletest::test]
fn test_account_not_found() {
    let account = "unknown_account".to_string();
    let data = "my_account - username: user123, password: pass456";

    let credential = Parser::new(account.clone());
    let result = credential.get_credentials(data.to_string());

    expect_that!(
        result,
        err(matches_pattern!(ParsingError::AccountNotFound(_)))
    );
}

#[googletest::test]
fn test_malformed_credentials() {
    let account = "my_account".to_string();
    let data = "my_account - username user123, password pass456";

    let credential = Parser::new(account.clone());
    let result = credential.get_credentials(data.to_string());

    expect_that!(
        result,
        ok(matches_pattern!(Credential {
            account: eq(&account),
            username: eq(""),
            password: eq("")
        }))
    )
}

#[googletest::test]
fn test_multiple_accounts() {
    let account = "account2".to_string();
    let data = "account1 - username: user1, password: pass1\n\
                account2 - username: user2, password: pass2\n\
                account3 - username: user3, password: pass3";

    let credential = Parser::new(account.clone());
    let result = credential.get_credentials(data.to_string());

    expect_that!(
        result,
        ok(eq(&Credential {
            account: account,
            username: "user2".to_string(),
            password: "pass2".to_string()
        }))
    );
}
