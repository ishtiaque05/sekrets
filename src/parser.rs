use crate::credentials::Credential;
#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error("No credentials found for account: `{0}'")]
    AccountNotFound(String),
}

#[derive(Debug, Default)]
pub struct Parser {
    pub account: String,
}

impl Parser {
    pub fn new(account: String) -> Self {
        Self { account }
    }

    pub fn get_credentials(&self, data: String) -> Result<Vec<Credential>, ParsingError> {
        let mut credentials = Vec::new();

        for line in data.lines() {
            let line = line.trim();

            let prefix = format!("{} - ", self.account);
            if line.starts_with(&prefix) {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() != 2 {
                    continue;
                }

                let credentials_part = parts[1];

                let mut username = String::new();
                let mut password = String::new();

                for pair in credentials_part.split(", ") {
                    if let Some(value) = pair.strip_prefix("username:") {
                        username = value.trim().to_string();
                    } else if let Some(value) = pair.strip_prefix("password:") {
                        password = value.trim().to_string();
                    }
                }

                credentials.push(Credential {
                    account: self.account.clone(),
                    username,
                    password,
                });
            }
        }

        if credentials.is_empty() {
            Err(ParsingError::AccountNotFound(self.account.clone()))
        } else {
            Ok(credentials)
        }
    }
}

#[cfg(test)]
mod tests;
