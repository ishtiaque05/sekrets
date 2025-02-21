use std::{fs, path::Path};

use clap::{Parser, Subcommand};

use crate::{
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
        #[arg(short, long, required = true)]
        accounts: Vec<String>,
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
}

pub fn run(cli: Cli) -> Result<()> {
    paths::ensure_dirs();

    match cli.command {
        Commands::Encrypt { file } => handle_encrypt(&file),
        Commands::Decrypt { accounts } => handle_decrypt(&accounts),
        Commands::Copy { dest } => handle_copy(&dest),
        Commands::Append {
            accounts,
            usernames,
        } => handle_append(&accounts, &usernames),
    }
}

fn handle_encrypt(file: &str) -> Result<()> {
    println!("Encrypting file: {}", file);

    let password = prompt_user_password();
    let encrypted_file = encryptor::encrypt_file(file, password.as_str())?;

    println!("Encrypted file created: {}", encrypted_file);
    Ok(())
}

fn handle_decrypt(accounts: &[String]) -> Result<()> {
    let password = prompt_user_password();
    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);
    let decrypted_data = decryptor::decrypt_file(&encrypted_filepath.to_string_lossy(), &password)?;

    for account in accounts {
        let credentials =
            CredentialParser::new(account.clone()).get_credentials(decrypted_data.clone())?;
        println!(
            "Account: {} - Username: {}, Password: {}",
            account, credentials.username, credentials.password
        );
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
            println!("Enter password credential for account: {}", account);

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
        return "foo".to_string();
    }
    use rpassword::read_password;
    println!("Enter password: ");
    read_password().expect("Failed to read password")
}

#[cfg(test)]
pub fn prompt_user_password() -> String {
    "foo".to_string()
}

#[cfg(test)]
mod tests;
