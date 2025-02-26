mod cli;
mod credential_manager;
mod credentials;
mod decryptor;
mod encryptor;
mod credential_file_parser;
mod password_generator;
mod paths;
mod types;

use anyhow::Result;
use clap::Parser;
use cli::{run, Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();

    run(cli)
}

pub mod tests;
