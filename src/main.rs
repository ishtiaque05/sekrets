use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use parser::Credential;
use rpassword::read_password; 

mod decryptor;
mod encryptor;
mod types;
mod parser;

fn main() -> Result<()> {
    let matches = Command::new("sekret")
    .version("1.0")
    .author("Syed Ishtiaque Ahmad")
    .about("Sssshhh it's a secret!!")
    .subcommand_required(true)
    .arg_required_else_help(true)
    
    .subcommand(
        Command::new("encrypt")
            .about("Encrypt a file")
            .arg(
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
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_name("FILE")
                    .help("File to decrypt")
                    .required(true)
                    .action(ArgAction::Set),
            )
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
    .get_matches();

    match matches.subcommand() {
        Some(("encrypt", sub_matches)) => {
            let file = sub_matches
                .get_one::<String>("file")
                .expect("File is required");
            println!("Encrypting file: {}", file);
            println!("Enter password: ");
            let password = read_password().expect("Failed to read password");
            let encrypted_file = encryptor::encrypt_file(file, password.as_str())?;
            println!("Encrypted file created: {}", encrypted_file);
        }
        Some(("decrypt", sub_matches)) => {
            let file = sub_matches
                .get_one::<String>("file")
                .expect("File is required");
            let accounts: Vec<&String> = sub_matches
                .get_many::<String>("accounts")
                .expect("At least one account is required")
                .collect();

            println!("Decrypting file: {}", file);
            println!("Enter password: ");
            let password = read_password().expect("Failed to read password");
            let decrypted_data = decryptor::decrypt_file(file, password.as_str())?;
            for account in accounts {
                let result = Credential::new(account.to_string()).get_credentials(decrypted_data.clone())?;
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
