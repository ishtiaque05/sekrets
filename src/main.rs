mod cli;
mod decryptor;
mod encryptor;
mod parser;
mod paths;
mod types;
mod credentials;

use anyhow::Result;
use cli::{build_cli, run};

fn main() -> Result<()> {
    paths::ensure_dirs();

    let matches = build_cli().get_matches();
    run(&matches)
}

pub mod tests;