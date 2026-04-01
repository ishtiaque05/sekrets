use super::*;
use googletest::prelude::*;

#[googletest::test]
fn test_detect_legacy_format() {
    let data = "account - username: user1, password: pass1".to_string();
    let parser = CredentialFileParser::new(data);

    expect_that!(parser.is_legacy_format(), eq(true));
}

#[googletest::test]
fn test_detect_jsonl_format() {
    let data = r#"{"account":"github","username":"foo","password":"bar","ts":"2026-03-31T10:00:00Z","history":[]}"#.to_string();
    let parser = CredentialFileParser::new(data);

    expect_that!(parser.is_legacy_format(), eq(false));
}

#[googletest::test]
fn test_parse_legacy_format() {
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
fn test_parse_jsonl_format() {
    let line1 = r#"{"account":"github","username":"foo","password":"bar","ts":"2026-03-31T10:00:00Z","history":[]}"#;
    let line2 = r#"{"account":"bank","username":"baz","password":"secret","ts":"2026-03-30T10:00:00Z","history":[{"password":"old","ts":"2026-03-01T10:00:00Z"}]}"#;
    let data = format!("{}\n{}", line1, line2);

    let parser = CredentialFileParser::new(data);
    let result = parser.get_all_credentials();

    expect_that!(result.len(), eq(2));

    let github = result
        .get(&("github".to_string(), "foo".to_string()))
        .unwrap();
    expect_that!(github.password, eq("bar"));
    expect_that!(github.ts, eq("2026-03-31T10:00:00Z"));
    expect_that!(github.history.len(), eq(0));

    let bank = result
        .get(&("bank".to_string(), "baz".to_string()))
        .unwrap();
    expect_that!(bank.password, eq("secret"));
    expect_that!(bank.history.len(), eq(1));
    expect_that!(bank.history[0].password, eq("old"));
}

#[googletest::test]
fn test_parse_legacy_format_failure_wrong_format() {
    let data = "account: username: user1 password: pass1\n".to_string();
    let parser = CredentialFileParser::new(data);
    let result = parser.get_all_credentials();

    expect_pred!(result.is_empty())
}

#[googletest::test]
fn test_serialize_credentials_to_jsonl() {
    let mut creds: CredentialHashMap = HashMap::new();
    creds.insert(
        ("github".to_string(), "foo".to_string()),
        Credential::new("github".to_string(), "foo".to_string(), "bar".to_string()),
    );

    let jsonl = CredentialFileParser::serialize_to_jsonl(&creds);
    let lines: Vec<&str> = jsonl.lines().collect();

    expect_that!(lines.len(), eq(1));

    // Verify it's valid JSON and round-trips
    let parsed: Credential = serde_json::from_str(lines[0]).unwrap();
    expect_that!(parsed.account, eq("github"));
    expect_that!(parsed.password, eq("bar"));
}

#[googletest::test]
fn test_empty_data() {
    let parser = CredentialFileParser::new(String::new());
    let result = parser.get_all_credentials();

    expect_pred!(result.is_empty());
}
