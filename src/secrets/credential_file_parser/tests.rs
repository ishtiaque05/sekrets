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

    expect_that!(
        val,
        eq(&Credential {
            account: "account".to_string(),
            username: "user2".to_string(),
            password: "pass2".to_string()
        })
    )
}

#[googletest::test]
fn test_get_all_credentials_failure_wrong_format_file() {
    let data = "account: username: user1 password: pass1\n".to_string();

    let parser = CredentialFileParser::new(data);

    let result = parser.get_all_credentials();

    expect_pred!(result.is_empty())
}
