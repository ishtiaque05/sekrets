use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use parser::Credential;
use rpassword::read_password; 

mod decryptor;
mod encryptor;
mod types;
mod parser;

fn main() -> Result<()> {
    let matches = Command::new("My CLI App")
        .version("1.0")
        .author("Syed Ishtiaque Ahmad")
        .about("Sssshhh its a secret!!")
        .arg(
            Arg::new("encrypt")
                .long("encrypt")
                .value_name("FILE")
                .help("Encrypts the specified file")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("sekrets")
                .long("sekrets")
                .value_name("FILE")
                .help("Decrypts the encrypted file to reveal passwords")
                .action(ArgAction::Set),
                
        )
        .arg(
            Arg::new("keyword")
                .help("The keyword for which to retrieve the password, e.g., --github")
                .long_help("Specific keyword like --github to fetch the password for GitHub")
                .action(ArgAction::Append),
        )
        .get_matches();

    if let Some(file) = matches.get_one::<String>("encrypt") {
        println!("File to encrypt: {}", file);

        println!("Enter password: ");
        let password = read_password().expect("Failed to read password");
        
        let res = encryptor::encrypt_file(file, password.as_ref()).unwrap();

        println!("encrypted file is {:?}", res.clone());
        println!(
            "Decrypt file {:?}",
            decryptor::decrypt_file(res.as_str(), password.as_ref())
        );
    }

    if let Some(crendential_file) = matches.get_one::<String>("sekrets") {
        println!("Credentials file path: {}", crendential_file);

        if let Some(keywords) = matches.get_many::<String>("keyword") {
            for keyword in keywords {
                println!("Keyword is {}", keyword);

                println!("Enter password: ");
                let password = read_password().expect("Failed to read password");
                let data = decryptor::decrypt_file(crendential_file.as_str(), password.as_ref())?;

                let result = Credential::new(keyword.to_string()).get_credentials(data)?;
                println!("Account: {} - Username: {}, Password: {}", keyword, result.username, result.password);
            }
        }
    }

    Ok(())
}
