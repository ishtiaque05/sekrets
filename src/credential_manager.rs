use crate::{
    credential_file_parser::{CredentialFileParser, CredentialHashMap}, credentials::Credential, decryptor, encryptor, paths::get_encrypted_file_path, types::FileError
};

pub struct CredentialManager {
    master_password: String,
    pub credentials: CredentialHashMap,
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

    pub fn find_creds(&mut self, account: &str, username: &str) -> Option<&mut Credential> {
        self.credentials.get_mut(&(account.to_string(), username.to_string()))
    }

    pub fn save_credentials(&self) -> Result<(), FileError> {
        let updated_data: String = self
            .credentials
            .values()
            .map(|c| c.format_as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let _ = encryptor::encrypt_text(&updated_data, &self.master_password);
        
        Ok(())
    }
}
