use crate::interactive::{interactive_mode, prompt_user_password};
use anyhow::Result;
use sekrets_core::secrets::credential_manager::CredentialManager;

pub fn handle_update(account: String, username: String) -> Result<()> {
    let password = prompt_user_password();
    let mut credential_manager = CredentialManager::new(password)?;
    crate::cli::commands::util::check_and_migrate(&credential_manager)?;

    if let Some(cred) = credential_manager.find_creds(&account, &username) {
        println!(
            "Enter new password for account: {}, username: {}",
            account, username
        );

        let new_password = interactive_mode()?;

        cred.update_pass(new_password);
        credential_manager.save_credentials()?;
        println!("Password updated successfully!");
    } else {
        return Err(anyhow::anyhow!(
            "No credentials found for account: `{}` with username: `{}`",
            account,
            username
        ));
    }

    Ok(())
}
