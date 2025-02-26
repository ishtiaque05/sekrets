use std::{fs, path::Path};

use clap::{Parser, Subcommand};

use crate::{
    credential_file_parser::{CredentialFileParser, CredentialHashMap},
    credential_manager::CredentialManager,
    credentials::Credential,
    decryptor,
    encryptor::{self, ENCRYPTED_FILENAME},
    password_generator::{prompt_user_password, PasswordGenerationError, PasswordGenerator},
    paths::{self, get_encrypted_file_path},
};
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(author, version, about)]
#[cfg_attr(test, derive(Debug))]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
enum Commands {
    /// Encrypt a file
    Encrypt {
        /// File to encrypt
        #[arg(short, long)]
        file: String,
    },

    /// Decrypt a file and retrieve credentials
    Decrypt {
        /// Account(s) for which to retrieve credentials
        #[arg(short, long = "account", required = true)]
        accounts: Vec<String>,

        #[arg(short, long = "username", required = false)]
        usernames: Vec<String>,
    },

    /// Copy the encrypted file to a new location
    Copy {
        /// Destination directory
        #[arg(short, long)]
        dest: String,
    },

    /// Append new credentials to the encrypted file
    Append {
        /// Account names
        #[arg(short, long = "account", required = true)]
        accounts: Vec<String>,

        /// Corresponding usernames
        #[arg(short, long = "username", required = true)]
        usernames: Vec<String>,
    },

    Update {
        #[arg(short, long = "account", required = true)]
        account: String,
        #[arg(short, long = "username", required = true)]
        username: String,
    },

    Generate {
        #[arg(short = 'p', long = "password", default_value_t = false)]
        generate_flag: bool,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    paths::ensure_dirs();

    match cli.command {
        Commands::Encrypt { file } => handle_encrypt(&file),
        Commands::Decrypt {
            accounts,
            usernames,
        } => handle_decrypt(&accounts, &usernames),
        Commands::Copy { dest } => handle_copy(&dest),
        Commands::Append {
            accounts,
            usernames,
        } => handle_append(&accounts, &usernames),
        Commands::Update { account, username } => handle_update(account, username),
        Commands::Generate { generate_flag } => generate_strong_password(generate_flag),
    }
}

fn generate_strong_password(flag: bool) -> Result<()> {
    if flag {
        PasswordGenerator::interactive_mode().map_err(anyhow::Error::from)?;
        Ok(())
    } else {
        Err(PasswordGenerationError::NoChoiceSelected.into())
    }
}

fn handle_encrypt(file: &str) -> Result<()> {
    println!("Encrypting file: {}", file);

    let password = prompt_user_password();
    let encrypted_file = encryptor::encrypt_file(file, password.as_str())?;

    println!("Encrypted file created: {}", encrypted_file);
    Ok(())
}

fn handle_decrypt(accounts: &[String], usernames: &[String]) -> Result<()> {
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

fn print_credentials(accounts: &[String], usernames: Vec<Option<String>>) -> Result<()> {
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

fn handle_copy(dest_dir: &str) -> Result<()> {
    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);
    let destination_path = Path::new(dest_dir).join(ENCRYPTED_FILENAME);

    fs::copy(&encrypted_filepath, &destination_path)
        .context("Failed to copy the encrypted file")?;

    println!("Encrypted file copied to: {}", destination_path.display());
    Ok(())
}

fn handle_append(accounts: &[String], usernames: &[String]) -> Result<()> {
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

fn handle_update(account: String, username: String) -> Result<()> {
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

fn confirm_interactive_pass_mode() -> Result<String> {
    if std::env::var("TEST_MODE").is_ok() {
        return Ok("no".to_string());
    }
    let mut response = String::new();
    std::io::stdin().read_line(&mut response)?;

    Ok(response.trim().to_lowercase())
}

#[cfg(test)]
mod tests;
