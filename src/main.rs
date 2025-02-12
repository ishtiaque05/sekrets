mod decryptor;
mod encryptor;
mod types;
mod parser;
mod cli;
mod paths;

use anyhow::Result;
use cli::{build_cli, run};



fn main() -> Result<()> {
    paths::ensure_dirs();

    let matches = build_cli().get_matches();
    run(&matches)
}
