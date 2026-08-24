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
