use anyhow::Result;

use crate::{
    encryption::{decryptor, encryptor::ENCRYPTED_FILENAME},
    helpers::directories::get_encrypted_file_path,
    secrets::credential_file_parser::CredentialFileParser,
    secrets::password_generator::prompt_user_password,
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
    let password = prompt_user_password();
    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);

    let decrypted_data = decryptor::decrypt_file(&encrypted_filepath.to_string_lossy(), &password)?;
    let parser = CredentialFileParser::new(decrypted_data);

    for (account, username) in accounts.iter().zip(usernames.iter()) {
        match parser.get_credentials(username.clone(), account.to_string()) {
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
