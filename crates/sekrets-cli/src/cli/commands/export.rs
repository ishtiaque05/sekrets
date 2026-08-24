use std::path::Path;

use anyhow::Result;

use crate::encryption::decryptor::decrypt_file;
use crate::encryption::encryptor::ENCRYPTED_FILENAME;
use crate::helpers::directories::get_encrypted_file_path;
use crate::secrets::password_generator::prompt_user_password;

use super::util::confirm_overwrite;

pub fn handle_export(output: &str) -> Result<()> {
    let password = prompt_user_password();

    if Path::new(output).exists() && !confirm_overwrite(output) {
        println!("Export cancelled.");
        return Ok(());
    }

    let encrypted_path = get_encrypted_file_path(ENCRYPTED_FILENAME);
    let plaintext = decrypt_file(&encrypted_path.to_string_lossy(), &password)?;

    std::fs::write(output, &plaintext)?;
    println!("Exported secrets to {}", output);
    Ok(())
}
