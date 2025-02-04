mod decryptor;
mod encryptor;
mod types;
mod parser;
mod cli;

use anyhow::Result;
use cli::build_cli;
use parser::Credential;
use rpassword::read_password; 

fn main() -> Result<()> {
    let matches = build_cli().get_matches();

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
