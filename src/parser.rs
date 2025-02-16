use crate::{decryptor, encryptor::{encrypt_text, ENCRYPTED_FILENAME}, paths::get_encrypted_file_path, types::FileError};

#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error("Failed to parse credentials for account: `{0}'")]
    AccountNotFound(String),
}

#[derive(Debug, Default)]
pub struct Parser {
    pub account: String,
}

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

impl Parser {
    pub fn new(account: String) -> Self {
        Self { account }
    }

    pub fn get_credentials(&self, data: String) -> Result<Credential, ParsingError> {
        for line in data.lines() {
            let line = line.trim();

            let prefix = format!("{} - ", self.account);
            if line.starts_with(&prefix) {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() != 2 {
                    continue;
                }

                let credentials_part = parts[1];

                let mut username: String = "".into();
                let mut password: String = "".into();

                for pair in credentials_part.split(", ") {
                    if pair.starts_with("username:") {
                        username = pair.trim_start_matches("username:").trim().to_string();
                    } else if pair.starts_with("password:") {
                        password = pair.trim_start_matches("password:").trim().to_string();
                    }
                }

                return Ok(Credential { account: self.account.clone(), username, password });
            }
        }
        Err(ParsingError::AccountNotFound(self.account.clone()))
    }
}

#[cfg(test)]
mod tests;
