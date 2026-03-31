use std::collections::HashMap;

use crate::secrets::credentials::Credential;

pub type CredentialHashMap = HashMap<(String, String), Credential>;

#[derive(Debug, Default)]
pub struct CredentialFileParser {
    pub decrypted_data: String,
}

impl CredentialFileParser {
    pub fn new(decrypted_data: String) -> Self {
        Self { decrypted_data }
    }

    pub fn get_all_credentials(&self) -> CredentialHashMap {
        let mut credentials_map: CredentialHashMap = HashMap::new();

        for line in self.decrypted_data.lines() {
            let line = line.trim();
            if let Some((account, credentials_part)) = line.split_once(" - ") {
                let mut username = String::new();
                let mut password = String::new();

                for pair in credentials_part.split(", ") {
                    if let Some(value) = pair.strip_prefix("username:") {
                        username = value.trim().to_string();
                    } else if let Some(value) = pair.strip_prefix("password:") {
                        password = value.trim().to_string();
                    }
                }

                if !account.is_empty() && !username.is_empty() && !password.is_empty() {
                    credentials_map.insert(
                        (account.to_string(), username.to_string()),
                        Credential::new(account.to_string(), username, password),
                    );
                }
            }
        }

        credentials_map
    }
}

#[cfg(test)]
mod tests;
