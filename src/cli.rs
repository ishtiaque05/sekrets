use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::{
    decryptor,
    encryptor::{self, ENCRYPTED_FILENAME},
    parser::Credential,
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
        .subcommand(
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
        .subcommand(
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
                    Credential::new(account.to_string()).get_credentials(decrypted_data.clone())?;
                println!(
                    "Account: {} - Username: {}, Password: {}",
                    account, result.username, result.password
                );
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
