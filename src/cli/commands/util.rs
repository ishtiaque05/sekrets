use anyhow::Result;

pub fn confirm_interactive_pass_mode() -> Result<String> {
    // For testing purpose
    if std::env::var("TEST_PASSWORD_INTERACTIVE").is_ok() {
        Ok(std::env::var("TEST_PASSWORD_INTERACTIVE").unwrap())
    } else {
        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;

        Ok(response.trim().to_lowercase())
    }
}

pub fn confirm_overwrite(path: &str) -> bool {
    if let Ok(response) = std::env::var("TEST_CONFIRM_OVERWRITE") {
        return response.trim().to_lowercase() == "yes";
    }

    println!("File '{}' already exists. Overwrite? [y/N]", path);
    let mut response = String::new();
    if std::io::stdin().read_line(&mut response).is_err() {
        return false;
    }
    response.trim().to_lowercase() == "y" || response.trim().to_lowercase() == "yes"
}
