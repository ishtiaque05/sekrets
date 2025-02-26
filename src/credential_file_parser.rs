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
        let credentials_map = self.get_all_credentials(); // Get all credentials as HashMap

        if let Some(ref uname) = username {
            if let Some(credential) = credentials_map.get(&(account.clone(), uname.clone())) {
                return Ok(vec![credential.clone()]);
            } else {
                return Err(ParsingError::AccountWithUsernameNotFound(
                    account,
                    uname.clone(),
                ));
            }
        }

        let matching_credentials: Vec<Credential> = credentials_map
            .into_iter()
            .filter_map(|((acct, _), credential)| {
                if acct == account {
                    Some(credential)
                } else {
                    None
                }
            })
            .collect();

        if matching_credentials.is_empty() {
            return Err(ParsingError::AccountNotFound(account));
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
