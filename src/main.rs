use anyhow::Result;
use clap::Parser;
use sekrets::cli::{run, Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();

    run(cli)
}
