use std::{fs, path::Path};

use anyhow::{Context, Result};
use sekrets_core::{
    encryption::encryptor::ENCRYPTED_FILENAME, helpers::directories::get_encrypted_file_path,
};

pub fn handle_copy(dest_dir: &str) -> Result<()> {
    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);
    let destination_path = Path::new(dest_dir).join(ENCRYPTED_FILENAME);

    fs::copy(&encrypted_filepath, &destination_path)
        .context("Failed to copy the encrypted file")?;

    println!("Encrypted file copied to: {}", destination_path.display());
    Ok(())
}
