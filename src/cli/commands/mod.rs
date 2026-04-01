pub mod append;
pub mod copy;
pub mod decrypt;
pub mod encrypt;
pub mod export;
pub mod find;
pub mod generate;
pub mod import;
pub mod self_update;
pub mod update;
pub mod util;
pub mod version;
use super::Commands;
use anyhow::Result;

pub fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Encrypt { file } => encrypt::handle_encrypt(&file),
        Commands::Decrypt {
            accounts,
            usernames,
            history,
        } => decrypt::handle_decrypt(&accounts, &usernames, history),
        Commands::Copy { dest } => copy::handle_copy(&dest),
        Commands::Append {
            accounts,
            usernames,
        } => append::handle_append(&accounts, &usernames),
        Commands::Update { account, username } => update::handle_update(account, username),
        Commands::Generate { generate_flag } => generate::generate_strong_password(generate_flag),
        Commands::Find { account } => find::account(account),
        Commands::Export { output } => export::handle_export(&output),
        Commands::Import { file } => import::handle_import(&file),
        Commands::Version { list, switch } => version::handle_version(list, switch),
    }
}
