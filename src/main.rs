mod cli;
mod credentials;
mod decryptor;
mod encryptor;
mod parser;
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
