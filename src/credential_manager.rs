use crate::{
    credentials::Credential, decryptor, encryptor, parser::Parser as CredentialParser,
    paths::get_encrypted_file_path, types::FileError,
};

pub struct CredentialManager {
    password: String,
    credentials: Vec<Credential>,
}

impl CredentialManager {
    pub fn new(password: String) -> Result<Self, FileError> {
        let encrypted_filepath = get_encrypted_file_path(crate::encryptor::ENCRYPTED_FILENAME)
            .to_string_lossy()
            .to_string();
        let decrypted_data = decryptor::decrypt_file(&encrypted_filepath, &password)?;
        let parser = CredentialParser::new(decrypted_data);

        Ok(Self {
            password,
            credentials: parser.get_all_credentials(),
        })
    }

    /// Update a credential's password
    pub fn update_password(
        &mut self,
        account: &str,
        username: &str,
        new_password: &str,
    ) -> Result<(), FileError> {
        let mut found = false;

        for cred in &mut self.credentials {
            if cred.account == account && cred.username == username {
                cred.password = new_password.to_string();
                found = true;
            }
        }

        if !found {
            println!(
                "⚠️ No credentials found for account: {} and username: {}",
                account, username
            );
            return Ok(());
        }

        self.save_credentials()?;
        println!("✅ Password updated successfully!");
        Ok(())
    }

    /// Save credentials back to encrypted file
    fn save_credentials(&self) -> Result<(), FileError> {
        let updated_data: String = self
            .credentials
            .iter()
            .map(|c| c.format_as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let _ = encryptor::encrypt_text(&updated_data, &self.password);
        Ok(())
    }
}
