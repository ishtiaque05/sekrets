use crate::secrets::{
    credential_manager::CredentialManager, password_generator::prompt_user_password,
};
use anyhow::Result;

pub fn account(account: String) -> Result<()> {
    let master_pass = prompt_user_password();
    let creds = CredentialManager::new(master_pass)?;

    let found_matches = creds.find_all_by_account(&account);
    println!("Found `{account}` matches: {}", found_matches.len());

    println!("{:?}", found_matches);

    Ok(())
}
