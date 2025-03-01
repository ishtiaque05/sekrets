pub mod append;
pub mod copy;
pub mod decrypt;
pub mod encrypt;
pub mod generate;
pub mod update;
pub mod util;
pub mod find;
use super::Commands;
use anyhow::Result;

pub fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Encrypt { file } => encrypt::handle_encrypt(&file),
        Commands::Decrypt {
            accounts,
            usernames,
        } => decrypt::handle_decrypt(&accounts, &usernames),
        Commands::Copy { dest } => copy::handle_copy(&dest),
        Commands::Append {
            accounts,
            usernames,
        } => append::handle_append(&accounts, &usernames),
        Commands::Update { account, username } => update::handle_update(account, username),
        Commands::Generate { generate_flag } => generate::generate_strong_password(generate_flag),
        Commands::Find { account } => find::account(account)
    }
}
