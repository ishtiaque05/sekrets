use std::{fs, path::Path};

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::{
    credentials::Credential,
    decryptor,
    encryptor::{self, ENCRYPTED_FILENAME},
    parser::Parser,
    paths::{self, get_encrypted_file_path},
};
use anyhow::Result;

pub fn build_cli() -> Command {
    Command::new("sekret")
        .version("1.0")
        .author("Syed Ishtiaque Ahmad")
        .about("Sssshhh it's a secret!!")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .add_encrypt_cmd()
        .add_decrypt_cmd()
        .add_copy_cmd()
        .add_append_cmd()
}

trait SekretsCommand {
    fn add_encrypt_cmd(self) -> Self;
    fn add_decrypt_cmd(self) -> Self;
    fn add_copy_cmd(self) -> Self;
    fn add_append_cmd(self) -> Self;
}

impl SekretsCommand for Command {
    fn add_encrypt_cmd(self) -> Self {
        self.subcommand(
            Command::new("encrypt").about("Encrypt a file").arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_name("FILE")
                    .help("File to encrypt")
                    .required(true)
                    .action(ArgAction::Set),
            ),
        )
    }

    fn add_decrypt_cmd(self) -> Self {
        self.subcommand(
            Command::new("decrypt")
                .about("Decrypt a file and retrieve account credentials")
                .arg(
                    Arg::new("accounts")
                        .short('a')
                        .long("accounts")
                        .value_name("ACCOUNT")
                        .help("Account(s) for which to retrieve credentials (e.g., github bank)")
                        .required(true)
                        .action(ArgAction::Append),
                ),
        )
    }

    fn add_copy_cmd(self) -> Self {
        self.subcommand(
            Command::new("copy")
                .about("Copy the encrypted file to a new location")
                .arg(
                    Arg::new("dest")
                        .short('d')
                        .long("dest")
                        .value_name("DIR")
                        .help("Destination directory for the encrypted file")
                        .required(true)
                        .action(ArgAction::Set),
                ),
        )
    }

    fn add_append_cmd(self) -> Self {
        self.subcommand(
            Command::new("append")
                .about("Append new credentials to the encrypted file")
                .arg(
                    Arg::new("account")
                        .short('a')
                        .long("account")
                        .value_name("ACCOUNT")
                        .help("Account name")
                        .required(true)
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("username")
                        .short('u')
                        .long("username")
                        .value_name("USERNAME")
                        .help("Username associated with the account")
                        .required(true)
                        .action(ArgAction::Append),
                ),
        )
    }
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    paths::ensure_dirs();

    match matches.subcommand() {
        Some(("encrypt", sub_matches)) => {
            let file = sub_matches
                .get_one::<String>("file")
                .expect("File is required");
            println!("Encrypting file: {}", file);
            let password = prompt_user_password();
            let encrypted_file = encryptor::encrypt_file(file, password.as_str())?;
            println!("Encrypted file created: {}", encrypted_file);
        }
        Some(("decrypt", sub_matches)) => {
            let accounts: Vec<&String> = sub_matches
                .get_many::<String>("accounts")
                .expect("At least one account is required")
                .collect();
            let password = prompt_user_password();
            let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);
            let decrypted_data =
                decryptor::decrypt_file(&encrypted_filepath.to_string_lossy(), password.as_str())?;

            for account in accounts {
                let result =
                    Parser::new(account.to_string()).get_credentials(decrypted_data.clone())?;
                println!(
                    "Account: {} - Username: {}, Password: {}",
                    account, result.username, result.password
                );
            }
        }
        Some(("copy", sub_matches)) => {
            let dest_dir = sub_matches
                .get_one::<String>("dest")
                .expect("Destination directory is required");

            let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);
            let destination_path = Path::new(dest_dir).join(ENCRYPTED_FILENAME);

            fs::copy(&encrypted_filepath, &destination_path)?;
            println!("Encrypted file copied to: {}", destination_path.display());
        }
        Some(("append", sub_matches)) => {
            let accounts: Vec<&String> = sub_matches
                .get_many::<String>("account")
                .expect("At least one account is required")
                .collect();

            let usernames: Vec<&String> = sub_matches
                .get_many::<String>("username")
                .expect("Each account must have a username")
                .collect();
            
            if accounts.len() != usernames.len() {
                println!("Each account must have a corresponding username.");
                return Err(anyhow::anyhow!("Mismatched accounts and usernames"));
            }

            let mut credentials = Vec::new();
            for (account, username) in accounts.iter().zip(usernames.iter()) {
                println!("Enter password credential for account: {}", account);
                let password = prompt_user_password();
                credentials.push(Credential::new((*account).to_string(), (*username).to_string(), password));
            }

            println!("Enter master password to decrypt the already encrypted file to add this new credentials");
            let master_pass = prompt_user_password();

            for cred in credentials {
                cred.add_to_encrypted_file(&master_pass)?;
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub fn prompt_user_password() -> String {
    if std::env::var("TEST_MODE").is_ok() {
        return "foo".to_string();
    }
    use rpassword::read_password;
    println!("Enter password: ");
    read_password().expect("Failed to read password")
}

#[cfg(test)]
mod tests;
