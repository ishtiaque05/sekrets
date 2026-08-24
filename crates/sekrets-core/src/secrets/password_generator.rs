use rand::seq::SliceRandom;
use thiserror::Error;
use zxcvbn::zxcvbn;

const DEFAULT_PASSWORD_LENGTH: usize = 16;

#[derive(Error, Debug)]
#[cfg_attr(any(test, feature = "test-utils"), derive(PartialEq, Eq))]
pub enum PasswordGenerationError {
    #[error("Password Cannot be weak entrophy < 4")]
    IsWeak,
    #[error("Password generation is not selected")]
    NoChoiceSelected,
}

pub struct PasswordGenerator {
    length: Option<usize>,
}

impl PasswordGenerator {
    pub fn new(length: Option<usize>) -> Self {
        PasswordGenerator { length }
    }

    pub fn generate_random(&self) -> String {
        let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  0123456789!@#$%^&*()-_+="
            .chars()
            .collect();
        self.generate_from_charset(&charset)
    }

    pub fn generate_letters_symbols(&self) -> String {
        let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  !@#$%^&*()-_+="
            .chars()
            .collect();
        self.generate_from_charset(&charset)
    }

    pub fn generate_letters_numbers(&self) -> String {
        let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  0123456789"
            .chars()
            .collect();
        self.generate_from_charset(&charset)
    }

    pub fn generate_from_charset(&self, charset: &[char]) -> String {
        let mut rng = rand::thread_rng();
        (0..self.length.unwrap_or(DEFAULT_PASSWORD_LENGTH))
            .map(|_| *charset.choose(&mut rng).unwrap())
            .collect()
    }
}

/// Checks if a password is strong (entropy >= 4). Advisory only — never blocks a save.
pub fn is_password_strong(password: &str) -> bool {
    match zxcvbn(password, &[]) {
        Ok(result) => result.score() >= 4,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests;
