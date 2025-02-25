use std::{fs, path::Path};

use clap::{Parser, Subcommand};

use crate::{
    credential_manager::CredentialManager,
    credentials::Credential,
    decryptor,
    encryptor::{self, ENCRYPTED_FILENAME},
    parser::Parser as CredentialParser,
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
    let parser = CredentialParser::new(decrypted_data);

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

    let decrypted_data =
        decryptor::decrypt_file(&encrypted_filepath.to_string_lossy(), &master_pass)?;
    let new_data = generate_new_credentials(&decrypted_data, accounts, usernames)?;

    encryptor::encrypt_text(&new_data, &master_pass)?;
    println!("✅ Credentials successfully added!");

    Ok(())
}

fn handle_update(account: String, username: String) -> Result<()> {
    let password = prompt_user_password();
    let mut credential_manager = CredentialManager::new(password)?;

    println!(
        "Enter new password for account: {}, username: {}",
        account, username
    );
    let new_password = prompt_user_password();

    credential_manager
        .update_password(&account, &username, &new_password)
        .expect("update password to succeed!");

    Ok(())
}

fn generate_new_credentials(
    existing_data: &str,
    accounts: &[String],
    usernames: &[String],
) -> Result<String> {
    let mut new_data = existing_data.to_string();

    let credentials: Vec<Credential> = accounts
        .iter()
        .zip(usernames.iter())
        .map(|(account, username)| {
            println!(
                "Enter password credential for account: {}, username: {}",
                account, username
            );

            Credential::new(account.clone(), username.clone(), prompt_user_password())
        })
        .collect();

    for cred in &credentials {
        new_data.push_str(&format!("\n{}", cred.format_as_str()));
    }

    Ok(new_data)
}

#[cfg(not(test))]
pub fn prompt_user_password() -> String {
    if std::env::var("TEST_MODE").is_ok() {
        "foo".to_string()
    } else {
        use rpassword::read_password;
        println!("Enter password: ");
        read_password().expect("Failed to read password")
    }
}

#[cfg(test)]
pub fn prompt_user_password() -> String {
    "foo".to_string()
}

#[cfg(test)]
mod tests;
