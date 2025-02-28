use crate::{credential_manager::CredentialManager, password_generator::{prompt_user_password, PasswordGenerator}};
use anyhow::Result;

pub fn handle_update(account: String, username: String) -> Result<()> {
    let password = prompt_user_password();
    let mut credential_manager = CredentialManager::new(password)?;

    if let Some(cred) = credential_manager.find_creds(&account, &username) {
        println!(
            "Enter new password for account: {}, username: {}",
            account, username
        );

        let new_password = PasswordGenerator::interactive_mode()?;

        cred.update_pass(new_password);
    }

    credential_manager.save_credentials()?;
    println!("✅ Password updated successfully!");

    Ok(())
}