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