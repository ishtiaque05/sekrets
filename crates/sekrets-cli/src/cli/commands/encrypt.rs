use crate::secrets::password_generator::prompt_user_password;
use anyhow::Result;
use sekrets_core::encryption::encryptor;

pub fn handle_encrypt(file: &str) -> Result<()> {
    println!("Encrypting file: {}", file);

    let password = prompt_user_password();
    let encrypted_file = encryptor::encrypt_file(file, password.as_str())?;

    println!("Encrypted file created: {}", encrypted_file);
    Ok(())
}
