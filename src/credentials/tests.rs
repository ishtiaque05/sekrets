use super::*;
use googletest::prelude::*;

#[googletest::test]
fn test_format_as_str() {
    let credential = Credential::new("github".into(), "user123".into(), "pass123".into());
    let expected = "github - username: user123, password: pass123";

    expect_that!(credential.format_as_str(), eq(expected));
}

#[googletest::test]
fn test_credential_creation() {
    let cred = Credential::new(
        "GitHub".to_string(),
        "user123".to_string(),
        "securePass!".to_string(),
    );

    expect_that!(cred.account, eq(&"GitHub".to_string()));
    expect_that!(cred.username, eq(&"user123".to_string()));
    expect_that!(cred.password, eq(&"securePass!".to_string()));
}

#[googletest::test]
fn test_update_password() {
    let mut cred = Credential::new(
        "GitHub".to_string(),
        "user123".to_string(),
        "oldPass".to_string(),
    );

    cred.update_pass("newSecurePass!".to_string());

    expect_that!(cred.password, eq(&"newSecurePass!".to_string()));
}
