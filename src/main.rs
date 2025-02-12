mod decryptor;
mod encryptor;
mod types;
mod parser;
mod cli;
mod paths;

use anyhow::Result;
use cli::build_cli;
use encryptor::ENCRYPTED_FILENAME;
use parser::Credential;
use paths::get_encrypted_file_path;


fn main() -> Result<()> {
    paths::ensure_dirs();

    let matches = build_cli().get_matches();

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
            let decrypted_data = decryptor::decrypt_file(&encrypted_filepath.to_string_lossy().into_owned(), password.as_str())?;

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

pub fn prompt_user_password() -> String {
    if std::env::var("TEST_MODE").is_ok() {
        return "foo".to_string();
    }
    use rpassword::read_password;
    println!("Enter password: ");
    read_password().expect("Failed to read password")
}
