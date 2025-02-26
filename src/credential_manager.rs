use crate::{
    credential_file_parser::CredentialFileParser, credentials::Credential, decryptor, encryptor, paths::get_encrypted_file_path, types::FileError
};

pub struct CredentialManager {
    master_password: String,
    credentials: Vec<Credential>,
}

impl CredentialManager {
    pub fn new(master_password: String) -> Result<Self, FileError> {
        let encrypted_filepath = get_encrypted_file_path(crate::encryptor::ENCRYPTED_FILENAME)
            .to_string_lossy()
            .to_string();
        let decrypted_data = decryptor::decrypt_file(&encrypted_filepath, &master_password)?;
        let parser = CredentialFileParser::new(decrypted_data);

        Ok(Self {
            master_password,
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
        if let Some(cred) = self
            .credentials
            .iter_mut()
            .find(|c| c.account == account && c.username == username)
        {
            cred.password = new_password.to_string();
            self.save_credentials()?;
            println!("✅ Password updated successfully!");
        } else {
            println!(
                "⚠️ No credentials found for account: {} and username: {}",
                account, username
            );
        }

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
        let _ = encryptor::encrypt_text(&updated_data, &self.master_password);
        Ok(())
    }
}
