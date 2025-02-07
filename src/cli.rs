use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("sekret")
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

}


#[cfg(test)]
mod tests;