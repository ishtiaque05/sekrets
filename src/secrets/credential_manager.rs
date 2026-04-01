use chrono::Utc;

use crate::{
    encryption::{self, decryptor, encryptor},
    helpers::directories::get_encrypted_file_path,
    secrets::{
        credential_file_parser::{CredentialFileParser, CredentialHashMap},
        credentials::Credential,
    },
    types::{CredentialError, FileError},
};

pub struct CredentialManager {
    master_password: String,
    pub credentials: CredentialHashMap,
    pub needs_migration: bool,
}

impl CredentialManager {
    pub fn new(master_password: String) -> Result<Self, FileError> {
        let encrypted_filepath = get_encrypted_file_path(encryption::encryptor::ENCRYPTED_FILENAME)
            .to_string_lossy()
            .to_string();
        let decrypted_data = decryptor::decrypt_file(&encrypted_filepath, &master_password)?;
        let parser = CredentialFileParser::new(decrypted_data);
        let needs_migration = parser.is_legacy_format();
        let mut credentials = parser.get_all_credentials();

        // If legacy format, stamp credentials with current time
        if needs_migration {
            let now = Utc::now().to_rfc3339();
            for cred in credentials.values_mut() {
                if cred.ts.is_empty() {
                    cred.ts = now.clone();
                }
            }
        }

        Ok(Self {
            master_password,
            credentials,
            needs_migration,
        })
    }

    pub fn find_creds(&mut self, account: &str, username: &str) -> Option<&mut Credential> {
        self.credentials
            .get_mut(&(account.to_string(), username.to_string()))
    }

    pub fn find_any_creds_with(
        &self,
        username: Option<String>,
        account: String,
    ) -> Result<Vec<Credential>, CredentialError> {
        if let Some(ref uname) = username {
            if let Some(credential) = self.credentials.get(&(account.clone(), uname.clone())) {
                return Ok(vec![credential.clone()]);
            } else {
                return Err(CredentialError::AccountWithUsernameNotFound(
                    account,
                    uname.clone(),
                ));
            }
        }

        let matching_credentials: Vec<Credential> = self
            .credentials
            .iter()
            .filter_map(|((acct, _), credential)| {
                if acct.to_lowercase().contains(&account.to_lowercase()) {
                    Some(credential.clone())
                } else {
                    None
                }
            })
            .collect();

        if matching_credentials.is_empty() {
            return Err(CredentialError::AccountNotFound(account));
        }

        Ok(matching_credentials)
    }

    pub fn find_all_by_account(&self, account: &str) -> Vec<String> {
        self.credentials
            .keys()
            .filter_map(|(a, _)| {
                if a.to_lowercase().contains(&account.to_lowercase()) {
                    Some(a.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn save_credentials(&self) -> Result<(), FileError> {
        let updated_data = CredentialFileParser::serialize_to_jsonl(&self.credentials);
        encryptor::encrypt_text(&updated_data, &self.master_password)?;
        Ok(())
    }

    /// Perform migration: snapshot old file as v1, save in new JSONL format.
    pub fn migrate(&self) -> Result<(), FileError> {
        use crate::secrets::version_manager;

        let current_path = get_encrypted_file_path(encryption::encryptor::ENCRYPTED_FILENAME);
        if current_path.exists() {
            version_manager::snapshot_current(&current_path)
                .map_err(|e| FileError::FileWriteError(e.to_string()))?;
        }

        self.save_credentials()
    }
}

#[cfg(test)]
mod tests;
