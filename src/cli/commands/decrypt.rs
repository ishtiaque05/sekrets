use anyhow::Result;

use crate::secrets::{
    credential_manager::CredentialManager, password_generator::prompt_user_password,
};

pub fn handle_decrypt(accounts: &[String], usernames: &[String]) -> Result<()> {
    if usernames.is_empty() {
        let unames = vec![None; accounts.len()];
        print_credentials(accounts, unames).expect("to print credentails");
    } else {
        if accounts.len() != usernames.len() {
            return Err(anyhow::anyhow!("Mismatched accounts and usernames"));
        }

        let some_usernames = usernames.iter().map(|s| Some(s.clone())).collect();
        print_credentials(accounts, some_usernames).expect("to print credentails");
    }

    Ok(())
}

pub fn print_credentials(accounts: &[String], usernames: Vec<Option<String>>) -> Result<()> {
    let master_pass = prompt_user_password();
    let cred_manager = CredentialManager::new(master_pass)?;
    crate::cli::commands::util::check_and_migrate(&cred_manager)?;

    for (account, username) in accounts.iter().zip(usernames.iter()) {
        match cred_manager.find_any_creds_with(username.clone(), account.to_string()) {
            Ok(credentials) => {
                for cred in credentials {
                    println!(
                        "Account: {} - Username: {}, Password: {}",
                        cred.account, cred.username, cred.password
                    );
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    Ok(())
}
