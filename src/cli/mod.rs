use clap::{Parser, Subcommand};

use anyhow::Result;
pub mod commands;

#[derive(Parser)]
#[command(author, version, about)]
#[cfg_attr(test, derive(Debug))]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub enum Commands {
    /// Encrypt a file
    Encrypt {
        /// File to encrypt
        #[arg(short, long)]
        file: String,
    },

    /// Decrypt a file and retrieve credentials
    Decrypt {
        /// Account(s) for which to retrieve credentials
        #[arg(short, long = "account", required = true)]
        accounts: Vec<String>,

        #[arg(short, long = "username", required = false)]
        usernames: Vec<String>,

        /// Show password change history
        #[arg(long, default_value_t = false)]
        history: bool,
    },

    /// Copy the encrypted file to a new location
    Copy {
        /// Destination directory
        #[arg(short, long)]
        dest: String,
    },

    /// Append new credentials to the encrypted file
    Append {
        /// Account names
        #[arg(short, long = "account", required = true)]
        accounts: Vec<String>,

        /// Corresponding usernames
        #[arg(short, long = "username", required = true)]
        usernames: Vec<String>,
    },

    Update {
        #[arg(short, long = "account", required = true)]
        account: String,
        #[arg(short, long = "username", required = true)]
        username: String,
    },

    Generate {
        #[arg(short = 'p', long = "password", default_value_t = false)]
        generate_flag: bool,
    },

    Find {
        #[arg(short = 'a', long = "account", required = true)]
        account: String,
    },

    /// Export decrypted secrets to a file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    crate::helpers::directories::ensure_dirs();

    commands::handle_command(cli.command)
}

#[cfg(test)]
mod tests;
