use crate::{decryptor, encryptor::{encrypt_text, ENCRYPTED_FILENAME}, paths::get_encrypted_file_path, types::FileError};

#[derive(Debug, PartialEq)]
pub struct Credential {
    pub account: String,
    pub username: String,
    pub password: String,
}

impl Credential {
    pub fn new(account: String, username: String, password: String) -> Self {
        Self {
            account,
            username,
            password
        }
    }

    pub fn format_as_str(&self) -> String {
        format!(
            "{} - username: {}, password: {}",
            self.account, self.username, self.password
        )
    }

    pub fn add_to_encrypted_file(&self, password: &str) -> Result<(), FileError> {
        let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);

        if !encrypted_filepath.exists() {
            return Err(FileError::DoesnotExist(format!(
                "{} file does not exist! Use `encrypt` first.",
                encrypted_filepath.display()
            )));
        }

        let decrypted_data = decryptor::decrypt_file(&encrypted_filepath.to_string_lossy(), password)?;

        let new_entry = self.format_as_str();
        
        let updated_data = format!("{}\n{}", decrypted_data.trim(), new_entry);

        encrypt_text(&updated_data, password)?;

        println!("✅ Credentials successfully added!");
        Ok(())
    }
}

#[cfg(test)]
mod tests;