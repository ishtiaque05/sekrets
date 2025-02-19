use super::*;
use googletest::prelude::*;

#[googletest::test]
fn test_format_as_str() {
    let credential = Credential::new("github".into(), "user123".into(), "pass123".into());
    let expected = "github - username: user123, password: pass123";

    expect_that!(credential.format_as_str(), eq(expected));
}
