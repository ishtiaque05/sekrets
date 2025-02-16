use crate::credentials::Credential;

#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error("Failed to parse credentials for account: `{0}'")]
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

    pub fn get_credentials(&self, data: String) -> Result<Credential, ParsingError> {
        for line in data.lines() {
            let line = line.trim();

            let prefix = format!("{} - ", self.account);
            if line.starts_with(&prefix) {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() != 2 {
                    continue;
                }

                let credentials_part = parts[1];

                let mut username: String = "".into();
                let mut password: String = "".into();

                for pair in credentials_part.split(", ") {
                    if pair.starts_with("username:") {
                        username = pair.trim_start_matches("username:").trim().to_string();
                    } else if pair.starts_with("password:") {
                        password = pair.trim_start_matches("password:").trim().to_string();
                    }
                }

                return Ok(Credential {
                    account: self.account.clone(),
                    username,
                    password,
                });
            }
        }
        Err(ParsingError::AccountNotFound(self.account.clone()))
    }
}

#[cfg(test)]
mod tests;
