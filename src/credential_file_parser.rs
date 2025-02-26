use std::collections::HashMap;

use crate::credentials::Credential;

pub type CredentialHashMap = HashMap<(String, String), Credential>;

#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error("No credentials found for account: `{0}'")]
    AccountNotFound(String),

    #[error("No credentials found for account: `{0}' with username: `{1}'")]
    AccountWithUsernameNotFound(String, String),
}

#[derive(Debug, Default)]
pub struct CredentialFileParser {
    pub decrypted_data: String,
}

impl CredentialFileParser {
    pub fn new(decrypted_data: String) -> Self {
        Self { decrypted_data }
    }

    pub fn get_credentials(
        &self,
        username: Option<String>,
        account: String,
    ) -> Result<Vec<Credential>, ParsingError> {
        let credentials_map = self.get_all_credentials();
        let mut matching_credentials = Vec::new();

        for ((acct, uname), credential) in credentials_map.into_iter() {
            if acct == account && (username.is_none() || username == Some(uname)) {
                matching_credentials.push(credential);
            }
        }

        if matching_credentials.is_empty() {
            return match username {
                Some(u) => Err(ParsingError::AccountWithUsernameNotFound(account, u)),
                None => Err(ParsingError::AccountNotFound(account)),
            };
        }

        Ok(matching_credentials)
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
                        Credential {
                            account: account.to_string(),
                            username,
                            password,
                        },
                    );
                }
            }
        }

        credentials_map
    }
}

#[cfg(test)]
mod tests;
