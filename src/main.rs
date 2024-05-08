use clap::{Command, Arg, ArgAction};
use anyhow::Result;  // Import Result from anyhow for error handling

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
