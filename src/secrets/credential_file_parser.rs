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

    /// Returns true if the data is in the old flat-text format.
    /// JSONL format starts with '{', legacy format does not.
    pub fn is_legacy_format(&self) -> bool {
        let trimmed = self.decrypted_data.trim();
        if trimmed.is_empty() {
            return false;
        }
        !trimmed.starts_with('{')
    }

    pub fn get_all_credentials(&self) -> CredentialHashMap {
        if self.decrypted_data.trim().is_empty() {
            return HashMap::new();
        }

        if self.is_legacy_format() {
            self.parse_legacy()
        } else {
            self.parse_jsonl()
        }
    }

    fn parse_jsonl(&self) -> CredentialHashMap {
        let mut credentials_map: CredentialHashMap = HashMap::new();

        for line in self.decrypted_data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(cred) = serde_json::from_str::<Credential>(line) {
                credentials_map.insert(
                    (cred.account.clone(), cred.username.clone()),
                    cred,
                );
            }
        }

        credentials_map
    }

    fn parse_legacy(&self) -> CredentialHashMap {
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

    /// Serialize a CredentialHashMap to JSONL format (one JSON object per line).
    pub fn serialize_to_jsonl(credentials: &CredentialHashMap) -> String {
        credentials
            .values()
            .map(|c| serde_json::to_string(c).expect("Credential serialization should not fail"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests;
