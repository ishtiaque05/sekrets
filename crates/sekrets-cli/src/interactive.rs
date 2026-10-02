use sekrets_core::secrets::password_generator::{
    is_password_strong, PasswordGenerationError, PasswordGenerator,
};
use std::io::{self, Write};

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

#[cfg(test)]
pub fn prompt_user_password() -> String {
    std::env::var("USER_TEST_PASS").unwrap_or("foo".to_string())
}

#[cfg(not(test))]
fn read_usize() -> usize {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse().unwrap_or(0)
}

#[cfg(test)]
fn read_usize() -> usize {
    16
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

    let choice = if std::env::var("PASSWORD_GENERATOR_CHOICE").is_ok() {
        std::env::var("PASSWORD_GENERATOR_CHOICE")
            .ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(4)
    } else {
        io::stdout().flush().unwrap();
        read_usize()
    };

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

    eprintln!("\nGenerated Password: {}", password);

    if is_password_strong(&password) {
        eprintln!("✅ Your password is strong!");
        Ok(password)
    } else {
        eprintln!("⚠️ Warning: Your password is weak. Consider making it longer or more complex.");
        Err(PasswordGenerationError::IsWeak)
    }
}

#[cfg(test)]
mod tests;
