use rand::seq::SliceRandom;
use std::io::{self, Write};
use thiserror::Error;
use zxcvbn::zxcvbn;

const DEFAULT_PASSWORD_LENGTH: usize = 16;

#[derive(Error, Debug)]
pub enum PasswordGenerationError {
    #[error("Password Cannot be blank")]
    IsBlank,
    #[error("Password Cannot be weak entrophy < 4")]
    IsWeak,
    #[error("Password generation is not selected")]
    NoChoiceSelected,
}

pub struct PasswordGenerator {
    length: Option<usize>,
}

impl PasswordGenerator {
    /// Creates a new PasswordGenerator with a specific length
    pub fn new(length: Option<usize>) -> Self {
        PasswordGenerator { length }
    }

    /// Generates a random password with letters, numbers, and symbols
    pub fn generate_random(&self) -> String {
        let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  0123456789!@#$%^&*()-_+="
            .chars()
            .collect();

        self.generate_from_charset(&charset)
    }

    /// Generates a password with only letters and symbols (no numbers)
    pub fn generate_letters_symbols(&self) -> String {
        let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  !@#$%^&*()-_+="
            .chars()
            .collect();

        self.generate_from_charset(&charset)
    }

    /// Generates a password with only letters and numbers (no symbols)
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

        let password = (0..self.length.unwrap_or(DEFAULT_PASSWORD_LENGTH))
            .map(|_| *charset.choose(&mut rng).unwrap())
            .collect();

        password
    }

    pub fn interactive_mode() -> Result<String, PasswordGenerationError> {
        if std::env::var("TEST_MODE").is_ok() {
            return Ok("bar".to_string());
        }

        println!("\nChoose password type:");
        println!("1) Random password (letters, numbers, symbols)");
        println!("2) Random letters & symbols only");
        println!("3) Random letters & numbers only");
        println!("4) Enter your own password");

        print!("Enter choice (1-5): ");
        io::stdout().flush().unwrap();
        let choice = read_usize();

        let password_generator = if choice != 4 {
            println!("Enter password length: ");
            let length = read_usize();
            PasswordGenerator::new(Some(length))
        } else {
            PasswordGenerator::new(None)
        };

        let password = match choice {
            1 => password_generator.generate_random(),
            2 => password_generator.generate_letters_symbols(),
            3 => password_generator.generate_letters_numbers(),
            4 => prompt_user_password(),
            _ => {
                println!("Invalid choice! Exiting.");
                return Err(PasswordGenerationError::NoChoiceSelected);
            }
        };

        println!("\nGenerated Password: {}", password);

        if password.is_empty() {
            return Err(PasswordGenerationError::IsBlank);
        }

        match is_password_strong(&password) {
            true => {
                println!("✅ Your password is strong!");
                Ok(password)
            }
            false => {
                println!(
                    "⚠️ Warning: Your password is weak. Consider making it longer or more complex."
                );
                Err(PasswordGenerationError::IsWeak)
            }
        }
    }
}

/// Checks if a password is strong (entropy ≥ 4)
fn is_password_strong(password: &str) -> bool {
    match zxcvbn(password, &[]) {
        Ok(result) => result.score() >= 4,
        Err(_) => false,
    }
}

/// Reads an integer (usize) from user input
fn read_usize() -> usize {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse().unwrap_or(0)
}

#[cfg(not(test))]
pub fn prompt_user_password() -> String {
    if std::env::var("TEST_MODE").is_ok() {
        "foo".to_string()
    } else {
        use rpassword::read_password;
        print!("Enter your password: ");
        io::stdout().flush().unwrap();
        read_password().expect("Failed to read password")
    }
}

/// Test mode: Returns a dummy password instead of asking for input
#[cfg(test)]
pub fn prompt_user_password() -> String {
    "foo".to_string()
}

#[cfg(test)]
mod tests;
