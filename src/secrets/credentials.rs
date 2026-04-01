use chrono::Utc;
use serde::{Deserialize, Serialize};

const MAX_HISTORY: usize = 5;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct HistoryEntry {
    pub password: String,
    pub ts: String,
}

impl HistoryEntry {
    pub fn format_ts_local(&self) -> String {
        use chrono::{DateTime, Local};
        if let Ok(utc) = self.ts.parse::<DateTime<Utc>>() {
            let local: DateTime<Local> = utc.into();
            local.format("%Y-%m-%d %I:%M %p %Z").to_string()
        } else {
            self.ts.clone()
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Credential {
    pub account: String,
    pub username: String,
    pub password: String,
    pub ts: String,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

impl Credential {
    pub fn new(account: String, username: String, password: String) -> Self {
        Self {
            account,
            username,
            password,
            ts: Utc::now().to_rfc3339(),
            history: Vec::new(),
        }
    }

    pub fn update_pass(&mut self, new_pass: String) {
        let old_entry = HistoryEntry {
            password: self.password.clone(),
            ts: self.ts.clone(),
        };

        // Insert at front (newest first)
        self.history.insert(0, old_entry);

        // Cap at MAX_HISTORY
        self.history.truncate(MAX_HISTORY);

        self.password = new_pass;
        self.ts = Utc::now().to_rfc3339();
    }

    pub fn format_as_str(&self) -> String {
        format!(
            "{} - username: {}, password: {}",
            self.account, self.username, self.password
        )
    }

    /// Format the timestamp in the user's local timezone for display.
    pub fn format_ts_local(&self) -> String {
        use chrono::{DateTime, Local};
        if let Ok(utc) = self.ts.parse::<DateTime<Utc>>() {
            let local: DateTime<Local> = utc.into();
            local.format("%Y-%m-%d %I:%M %p %Z").to_string()
        } else {
            self.ts.clone()
        }
    }
}

#[cfg(test)]
mod tests;
