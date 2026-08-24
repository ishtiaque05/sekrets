use super::*;
use googletest::prelude::*;
use temp_env::with_vars;

#[googletest::test]
fn test_interactive_mode_in_test_mode() {
    with_vars(
        vec![
            ("PASSWORD_GENERATOR_CHOICE", Some("4")),
            ("USER_TEST_PASS", Some("A^u4IfqU#PRla8+e")),
        ],
        || {
            let result = interactive_mode();
            expect_that!(result.unwrap(), eq(&"A^u4IfqU#PRla8+e".to_string()));
        },
    );
}

#[googletest::test]
fn test_interactive_mode_weak_pass_opt_4() {
    with_vars(
        vec![
            ("PASSWORD_GENERATOR_CHOICE", Some("4")),
            ("USER_TEST_PASS", Some("foo")),
        ],
        || {
            let result = interactive_mode();
            expect_that!(
                result,
                err(matches_pattern!(PasswordGenerationError::IsWeak))
            );
        },
    );
}

#[googletest::test]
fn test_interactive_mode_weak_invalid_opt() {
    with_vars(
        vec![
            ("PASSWORD_GENERATOR_CHOICE", Some("5")),
            ("USER_TEST_PASS", Some("foo")),
        ],
        || {
            let result = interactive_mode();
            expect_that!(
                result,
                err(matches_pattern!(PasswordGenerationError::NoChoiceSelected))
            );
        },
    );
}
