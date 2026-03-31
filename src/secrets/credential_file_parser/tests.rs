use super::*;
use googletest::prelude::*;

#[googletest::test]
fn test_get_all_credentials() {
    let data = "account - username: user1, password: pass1\n\
                account - username: user2, password: pass2\n\
                account3 - username: user3, password: pass3"
        .to_string();

    let parser = CredentialFileParser::new(data);

    let result = parser.get_all_credentials();

    expect_that!(result.len(), eq(3));
    let val = result
        .get(&("account".to_string(), "user2".to_string()))
        .unwrap();

    expect_that!(val.account, eq("account"));
    expect_that!(val.username, eq("user2"));
    expect_that!(val.password, eq("pass2"));
}

#[googletest::test]
fn test_get_all_credentials_failure_wrong_format_file() {
    let data = "account: username: user1 password: pass1\n".to_string();

    let parser = CredentialFileParser::new(data);

    let result = parser.get_all_credentials();

    expect_pred!(result.is_empty())
}
