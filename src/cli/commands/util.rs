use anyhow::Result;
use crate::secrets::credential_manager::CredentialManager;

/// Check if migration is needed and prompt user.
/// Returns Ok(true) if migration was performed, Ok(false) if not needed.
pub fn check_and_migrate(manager: &CredentialManager) -> Result<bool> {
    if !manager.needs_migration {
        return Ok(false);
    }

    println!("Your sekrets file uses an older format. It will be upgraded to the new format.");
    println!("A backup of your current file will be saved before migrating.");
    print!("Proceed? (y/n): ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let response = if std::env::var("TEST_MODE").is_ok() || cfg!(test) {
        "y".to_string()
    } else {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_lowercase()
    };

    if response == "y" || response == "yes" {
        manager.migrate().map_err(|e| anyhow::anyhow!(e))?;
        println!("Migration complete. Backup saved to versions/sekrets.v1.enc");
        Ok(true)
    } else {
        println!("Migration cancelled. Some features may not work correctly.");
        Ok(false)
    }
}

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
