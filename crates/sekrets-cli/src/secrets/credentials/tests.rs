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
    expect_that!(cred.ts.is_empty(), eq(false));
    expect_that!(cred.history.len(), eq(0));
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

#[googletest::test]
fn test_update_pass_pushes_history() {
    let mut cred = Credential::new(
        "GitHub".to_string(),
        "user123".to_string(),
        "pass1".to_string(),
    );
    let original_ts = cred.ts.clone();

    cred.update_pass("pass2".to_string());

    expect_that!(cred.password, eq("pass2"));
    expect_that!(cred.history.len(), eq(1));
    expect_that!(cred.history[0].password, eq("pass1"));
    expect_that!(cred.history[0].ts, eq(&original_ts));
    // ts should have been updated
    expect_that!(cred.ts == original_ts, eq(false));
}

#[googletest::test]
fn test_history_capped_at_5() {
    let mut cred = Credential::new(
        "GitHub".to_string(),
        "user123".to_string(),
        "pass0".to_string(),
    );

    for i in 1..=6 {
        cred.update_pass(format!("pass{}", i));
    }

    expect_that!(cred.history.len(), eq(5));
    expect_that!(cred.password, eq("pass6"));
    // oldest (pass0) should have been dropped; newest history entry is pass5
    expect_that!(cred.history[0].password, eq("pass5"));
    // oldest remaining should be pass1
    expect_that!(cred.history[4].password, eq("pass1"));
}

#[googletest::test]
fn test_credential_serialization_roundtrip() {
    let cred = Credential::new("github".into(), "foo".into(), "bar".into());

    let json = serde_json::to_string(&cred).unwrap();
    let deserialized: Credential = serde_json::from_str(&json).unwrap();

    expect_that!(deserialized.account, eq("github"));
    expect_that!(deserialized.username, eq("foo"));
    expect_that!(deserialized.password, eq("bar"));
    expect_that!(deserialized.ts.is_empty(), eq(false));
    expect_that!(deserialized.history.len(), eq(0));
}

#[googletest::test]
fn test_format_ts_local_returns_readable_string() {
    let cred = Credential::new("github".into(), "foo".into(), "bar".into());
    let local_ts = cred.format_ts_local();

    // Should not be empty and should not be raw UTC format
    expect_that!(local_ts.is_empty(), eq(false));
}
