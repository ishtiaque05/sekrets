mod cli;
mod credentials;
mod decryptor;
mod encryptor;
mod parser;
mod paths;
mod types;

use anyhow::Result;
use cli::{build_cli, run};

fn main() -> Result<()> {
    paths::ensure_dirs();

    let matches = build_cli().get_matches();
    run(&matches)
}

pub mod tests;
