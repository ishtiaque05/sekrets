use super::*;
use googletest::prelude::*;
use temp_env::with_vars;

#[googletest::test]
fn test_generate_random() {
    let generator = PasswordGenerator::new(Some(16));
    let password = generator.generate_random();

    expect_that!(password.len(), eq(16));
    expect_that!(
        password.chars().all(|c| {
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_+="
                .contains(c)
        }),
        eq(true)
    );
}

#[googletest::test]
fn test_generate_letters_symbols() {
    let generator = PasswordGenerator::new(Some(16));
    let password = generator.generate_letters_symbols();

    expect_that!(password.len(), eq(16));
    expect_that!(
        password.chars().all(|c| {
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()-_+=".contains(c)
        }),
        eq(true)
    );
}

#[googletest::test]
fn test_generate_letters_numbers() {
    let generator = PasswordGenerator::new(Some(16));
    let password = generator.generate_letters_numbers();

    expect_that!(password.len(), eq(16));
    expect_that!(
        password
            .chars()
            .all(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".contains(c)),
        eq(true)
    );
}

#[googletest::test]
fn test_is_password_strong() {
    expect_that!(is_password_strong("VerySecureP@ssw0rd!123"), eq(true));
    expect_that!(is_password_strong("weak"), eq(false));
}

#[googletest::test]
fn test_interactive_mode_in_test_mode() {
    with_vars(
        vec![
            ("PASSWORD_GENERATOR_CHOICE", Some("4")),
            ("USER_TEST_PASS", Some("A^u4IfqU#PRla8+e")),
        ],
        || {
            let result = PasswordGenerator::interactive_mode();
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
            let result = PasswordGenerator::interactive_mode();
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
            let result = PasswordGenerator::interactive_mode();
            expect_that!(
                result,
                err(matches_pattern!(PasswordGenerationError::NoChoiceSelected))
            );
        },
    );
}
