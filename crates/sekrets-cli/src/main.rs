use anyhow::Result;
use clap::Parser;
use sekrets_cli::cli::{run, Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();

    run(cli)
}
