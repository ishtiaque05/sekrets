use std::path::PathBuf;

use thiserror::Error;

use crate::encryption::{
    decryptor,
    encryptor::{self, ENCRYPTED_FILENAME},
};
use crate::helpers::directories::get_encrypted_file_path;
use crate::secrets::credential_manager::CredentialManager;
use crate::secrets::credentials::Credential;
use crate::secrets::credentials::HistoryEntry;
use crate::secrets::version_manager::{self, VersionInfo};
use crate::types::{CredentialError, FileError};

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("No sekrets file found at {0}")]
    FileNotFound(PathBuf),
    #[error("Incorrect master password")]
    WrongPassword,
    #[error("Sekrets file is corrupt: {0}")]
    Corrupt(String),
    #[error("No credentials found for account: `{account}'")]
    AccountNotFound { account: String },
    #[error("A credential already exists for account `{account}' with username `{username}'")]
    AccountAlreadyExists { account: String, username: String },
    #[error("No credentials found for account: `{account}' with username: `{username}'")]
    AccountWithUsernameNotFound { account: String, username: String },
    #[error("I/O error: {0}")]
    Io(String),
}

impl From<FileError> for VaultError {
    fn from(err: FileError) -> Self {
        match err {
            FileError::InvalidCiphertext(msg) => VaultError::Corrupt(msg),
            FileError::FileReadError(msg) | FileError::FileWriteError(msg) => VaultError::Io(msg),
            FileError::HashingError(_)
            | FileError::InvalidHashOutput(_)
            | FileError::InvalidNonceSize(_)
            | FileError::EncryptionError(_)
            | FileError::DecryptionError(_)
            | FileError::KeyGenerationError(_) => VaultError::WrongPassword,
        }
    }
}

impl From<CredentialError> for VaultError {
    fn from(err: CredentialError) -> Self {
        match err {
            CredentialError::AccountNotFound(account) => VaultError::AccountNotFound { account },
            CredentialError::AccountWithUsernameNotFound(account, username) => {
                VaultError::AccountWithUsernameNotFound { account, username }
            }
        }
    }
}

#[derive(Debug)]
pub struct Vault {
    manager: CredentialManager,
}

impl Vault {
    pub fn locate() -> Option<PathBuf> {
        let path = get_encrypted_file_path(ENCRYPTED_FILENAME);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn unlock(master_password: &str) -> Result<Vault, VaultError> {
        let path = get_encrypted_file_path(ENCRYPTED_FILENAME);
        if !path.exists() {
            return Err(VaultError::FileNotFound(path));
        }

        let manager = CredentialManager::new(master_password.to_string())?;
        Ok(Vault { manager })
    }

    pub fn create(master_password: &str) -> Result<Vault, VaultError> {
        let path = get_encrypted_file_path(ENCRYPTED_FILENAME);
        if path.exists() {
            return Err(VaultError::Io(format!(
                "{} already exists",
                path.display()
            )));
        }

        encryptor::encrypt_text("", master_password)?;
        Vault::unlock(master_password)
    }

    pub fn needs_migration(&self) -> bool {
        self.manager.needs_migration
    }

    pub fn migrate(&mut self) -> Result<(), VaultError> {
        self.manager.migrate()?;
        self.manager.needs_migration = false;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<&Credential> {
        if query.is_empty() {
            return self.manager.credentials.values().collect();
        }
        let q = query.to_lowercase();
        self.manager
            .credentials
            .values()
            .filter(|c| c.account.to_lowercase().contains(&q) || c.username.to_lowercase().contains(&q))
            .collect()
    }

    pub fn add(&mut self, account: &str, username: &str, password: &str) -> Result<(), VaultError> {
        let key = (account.to_string(), username.to_string());
        if self.manager.credentials.contains_key(&key) {
            return Err(VaultError::AccountAlreadyExists {
                account: account.to_string(),
                username: username.to_string(),
            });
        }
        self.manager.credentials.insert(
            key,
            Credential::new(account.to_string(), username.to_string(), password.to_string()),
        );
        self.save()
    }

    pub fn update(&mut self, account: &str, username: &str, new_password: &str) -> Result<(), VaultError> {
        let cred = self
            .manager
            .find_creds(account, username)
            .ok_or_else(|| VaultError::AccountWithUsernameNotFound {
                account: account.to_string(),
                username: username.to_string(),
            })?;
        cred.update_pass(new_password.to_string());
        self.save()
    }

    pub fn delete(&mut self, account: &str, username: &str) -> Result<(), VaultError> {
        let key = (account.to_string(), username.to_string());
        if self.manager.credentials.remove(&key).is_none() {
            return Err(VaultError::AccountWithUsernameNotFound {
                account: account.to_string(),
                username: username.to_string(),
            });
        }
        self.save()
    }

    pub fn save(&self) -> Result<(), VaultError> {
        self.manager.save_credentials().map_err(VaultError::from)
    }

    pub fn history(&self, account: &str, username: &str) -> Result<&[HistoryEntry], VaultError> {
        self.manager
            .credentials
            .get(&(account.to_string(), username.to_string()))
            .map(|c| c.history.as_slice())
            .ok_or_else(|| VaultError::AccountWithUsernameNotFound {
                account: account.to_string(),
                username: username.to_string(),
            })
    }

    pub fn change_master_password(&mut self, new_password: &str) -> Result<(), VaultError> {
        self.manager.set_master_password(new_password.to_string());
        self.save()
    }

    pub fn list_versions(&self) -> Result<Vec<VersionInfo>, VaultError> {
        version_manager::list_versions().map_err(VaultError::from)
    }

    pub fn switch_version(&mut self, n: usize, version_password: &str) -> Result<(), VaultError> {
        let version_path = version_manager::get_version_file_path(n);
        if !version_path.exists() {
            return Err(VaultError::FileNotFound(version_path));
        }

        let version_path_str = version_path
            .to_str()
            .ok_or_else(|| VaultError::Io("invalid version path".to_string()))?
            .to_string();
        let version_data = decryptor::decrypt_file(&version_path_str, version_password)?;

        let current_path = get_encrypted_file_path(ENCRYPTED_FILENAME);
        if current_path.exists() {
            version_manager::snapshot_current(&current_path)?;
        }

        let current_master_password = self.manager.master_password().to_string();
        encryptor::encrypt_text(&version_data, &current_master_password)?;

        let refreshed = Vault::unlock(&current_master_password)?;
        *self = refreshed;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
