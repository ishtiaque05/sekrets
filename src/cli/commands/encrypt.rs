use crate::{encryptor, password_generator::prompt_user_password};
use anyhow::Result;

pub fn handle_encrypt(file: &str) -> Result<()> {
    println!("Encrypting file: {}", file);

    let password = prompt_user_password();
    let encrypted_file = encryptor::encrypt_file(file, password.as_str())?;

    println!("Encrypted file created: {}", encrypted_file);
    Ok(())
}