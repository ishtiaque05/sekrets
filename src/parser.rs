use crate::credentials::Credential;

#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error("No credentials found for account: `{0}'")]
    AccountNotFound(String),

    #[error("No credentials found for account: `{0}' with username: `{1}'")]
    AccountWithUsernameNotFound(String, String),
}

#[derive(Debug, Default)]
pub struct Parser {
    pub decrypted_data: String,
}

impl Parser {
    pub fn new(decrypted_data: String) -> Self {
        Self { decrypted_data }
    }

    pub fn get_credentials(
        &self,
        username: Option<String>,
        account: String,
    ) -> Result<Vec<Credential>, ParsingError> {
        let mut credentials = Vec::new();

        for line in self.decrypted_data.lines() {
            let line = line.trim();

            let prefix = format!("{} - ", account);
            if line.starts_with(&prefix) {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() != 2 {
                    continue;
                }

                let credentials_part = parts[1];

                let mut matched_username = String::new();
                let mut password = String::new();

                for pair in credentials_part.split(", ") {
                    if let Some(value) = pair.strip_prefix("username:") {
                        matched_username = value.trim().to_string();
                    } else if let Some(value) = pair.strip_prefix("password:") {
                        password = value.trim().to_string();
                    }
                }

                // Only push if there's no username provided or it matches
                if username.as_ref().map_or(true, |u| u == &matched_username) {
                    credentials.push(Credential {
                        account: account.clone(),
                        username: matched_username.clone(), // Keep the found username
                        password,
                    });
                }
            }
        }

        // Borrow `username` here instead of consuming it
        match (credentials.is_empty(), &username) {
            (true, Some(username)) => Err(ParsingError::AccountWithUsernameNotFound(
                account.clone(),
                username.clone(),
            )),
            (true, None) => Err(ParsingError::AccountNotFound(account.clone())),
            _ => Ok(credentials),
        }
    }
}

#[cfg(test)]
mod tests;
