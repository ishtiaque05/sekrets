use std::env;

use super::*;
use googletest::prelude::*;

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
    env::remove_var("TEST_MODE");
    env::set_var("PASSWORD_GENERATOR_CHOICE", "4");
    env::set_var("USER_TEST_PASS", "A^u4IfqU#PRla8+e");

    let result = PasswordGenerator::interactive_mode();
    expect_that!(result.unwrap(), eq(&"A^u4IfqU#PRla8+e".to_string()));

    env::remove_var("PASSWORD_GENERATOR_CHOICE");
    env::remove_var("USER_TEST_PASS");
}

#[googletest::test]
fn test_interactive_mode_weak_pass_opt_4() {
    env::set_var("PASSWORD_GENERATOR_CHOICE", "4");
    env::set_var("USER_TEST_PASS", "foo");

    let result = PasswordGenerator::interactive_mode();
    expect_that!(
        result,
        err(matches_pattern!(PasswordGenerationError::IsWeak))
    );

    env::remove_var("PASSWORD_GENERATOR_CHOICE");
    env::remove_var("USER_TEST_PASS");
}

#[googletest::test]
fn test_interactive_mode_weak_invalid_opt() {
    env::set_var("PASSWORD_GENERATOR_CHOICE", "5");
    env::set_var("USER_TEST_PASS", "foo");

    let result = PasswordGenerator::interactive_mode();
    expect_that!(
        result,
        err(matches_pattern!(PasswordGenerationError::NoChoiceSelected))
    );

    env::remove_var("PASSWORD_GENERATOR_CHOICE");
    env::remove_var("USER_TEST_PASS");
}
