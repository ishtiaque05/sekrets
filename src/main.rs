use clap::{Arg, ArgAction, Command};
use anyhow::Result;  // Import Result from anyhow for error handling

mod encryptor;
mod decryptor;
mod types;

fn main() -> Result<()> {
    let matches = Command::new("My CLI App")
        .version("1.0")
        .author("Syed Ishtiaque Ahmad")
        .about("Sssshhh its a secret!!")
        .arg(Arg::new("encrypt")
             .long("encrypt")
             .value_name("FILE")
             .help("Encrypts the specified file")
             .action(ArgAction::Set))
        .arg(Arg::new("sekrets")
            .long("sekrets")
            .help("Decrypts the encrypted file to reveal passwords"))
       .arg(Arg::new("keyword")
            .help("The keyword for which to retrieve the password, e.g., --github")
            .long_help("Specific keyword like --github to fetch the password for GitHub")
            .action(ArgAction::Append))  // Changed to Append to allow multiple values
        .get_matches();

    if let Some(file) = matches.get_one::<String>("encrypt") {
        println!("File to encrypt: {}", file);
        let res = encryptor::encrypt_file(file, "foo").unwrap();
        
        println!("encrypted file is {:?}", res.clone());
        println!("Decrypt file {:?}", decryptor::decrypt_file(res.as_str(), "foo"));
    }

    if matches.contains_id("sekrets") {
        println!("Sekrets option selected");
    }

    if let Some(keywords) = matches.get_many::<String>("keyword") {
        for keyword in keywords {
            println!("Keyword: {}", keyword);
        }
    }
       

    Ok(())
}
