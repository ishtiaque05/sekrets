#[derive(Debug, PartialEq)]
pub struct Credential {
    pub account: String,
    pub username: String,
    pub password: String,
}

impl Credential {
    pub fn new(account: String, username: String, password: String) -> Self {
        Self {
            account,
            username,
            password,
        }
    }

    pub fn format_as_str(&self) -> String {
        format!(
            "{} - username: {}, password: {}",
            self.account, self.username, self.password
        )
    }
}

#[cfg(test)]
mod tests;
