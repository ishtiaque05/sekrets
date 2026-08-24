use crate::{
    cli::commands::util::confirm_interactive_pass_mode,
    encryption::encryptor::ENCRYPTED_FILENAME,
    helpers::directories::get_encrypted_file_path,
    secrets::{
        credential_file_parser::CredentialHashMap,
        credential_manager::CredentialManager,
        credentials::Credential,
        password_generator::{prompt_user_password, PasswordGenerator},
    },
};
use anyhow::Result;

pub fn handle_append(accounts: &[String], usernames: &[String]) -> Result<()> {
    if accounts.len() != usernames.len() {
        return Err(anyhow::anyhow!("Mismatched accounts and usernames"));
    }

    let master_pass = prompt_user_password();
    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);

    if !encrypted_filepath.exists() {
        return Err(anyhow::anyhow!(
            "{} does not exist! Encrypt file first",
            encrypted_filepath.display()
        ));
    }

    let mut credential_manager = CredentialManager::new(master_pass.clone())?;
    crate::cli::commands::util::check_and_migrate(&credential_manager)?;

    for (account, username) in accounts.iter().zip(usernames.iter()) {
        let key = (account.clone(), username.clone());

        if let Some(existing_cred) = credential_manager.credentials.get_mut(&key) {
            println!(
                "Credential already exists for account: {}, username: {}",
                account, username
            );
            println!("Do you want to update the password? (yes/no): ");

            let response = confirm_interactive_pass_mode()?;

            if response == "yes" {
                println!(
                    "Enter new password for account: {}, username: {}",
                    account, username
                );

                let new_pass = PasswordGenerator::interactive_mode().expect("new password");

                existing_cred.update_pass(new_pass);

                println!("✅ Password updated successfully!");
            } else {
                println!(
                    "Password update skipped for account {}, username: {}",
                    account, username
                )
            }
        } else {
            add_new_creds(account, username, &mut credential_manager.credentials);
        }
    }

    credential_manager.save_credentials()?;

    println!("✅ Changes successfully saved!");

    Ok(())
}

fn add_new_creds(account: &str, username: &str, new_credentials: &mut CredentialHashMap) {
    println!(
        "Adding new credential for account: {}, username: {}",
        account, username
    );
    let password = PasswordGenerator::interactive_mode().expect("interactive pass not to fail");

    new_credentials.insert(
        (account.to_string(), username.to_string()),
        Credential::new(account.to_string(), username.to_string(), password),
    );
}
