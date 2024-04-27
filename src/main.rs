use clap::{Command, Arg};
use anyhow::Result;  // Import Result from anyhow for error handling

fn main() -> Result<()> {
    let matches = Command::new("My CLI App")
        .version("1.0")
        .author("Syed Ishtiaque ahmad")
        .about("Sssshhh its a secret!!")
        .arg(Arg::new("config")
             .short('c')  // Ensure the 'short' method receives a char
             .long("config")
             .value_name("FILE")
             .help("Sets a custom config file")
             .action(clap::ArgAction::Set))  // Correct method to specify that this argument takes a value
        .get_matches();

    if let Some(config) = matches.get_one::<String>("config") {
        println!("Value for config: {}", config);
    } else {
        println!("No config file provided");
    }

    Ok(())
}
