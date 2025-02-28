pub mod encrypt;
pub mod decrypt;
pub mod copy;
pub mod append;
pub mod update;
pub mod generate;
pub mod util;

use anyhow::Result;
use super::Commands;

pub fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Encrypt { file } => encrypt::handle_encrypt(&file),
        Commands::Decrypt { accounts, usernames } => decrypt::handle_decrypt(&accounts, &usernames),
        Commands::Copy { dest } => copy::handle_copy(&dest),
        Commands::Append { accounts, usernames } => append::handle_append(&accounts, &usernames),
        Commands::Update { account, username } => update::handle_update(account, username),
        Commands::Generate { generate_flag } => generate::generate_strong_password(generate_flag),
    }
}