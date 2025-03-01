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
}

impl CredentialManager {
    pub fn new(master_password: String) -> Result<Self, FileError> {
        let encrypted_filepath = get_encrypted_file_path(encryption::encryptor::ENCRYPTED_FILENAME)
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
                if acct == &account {
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

#[cfg(test)]
mod tests;
