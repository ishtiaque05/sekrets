pub mod encryption;
pub mod helpers;
pub mod secrets;
pub mod types;
mod vault;

pub use secrets::credentials::{Credential, HistoryEntry};
pub use secrets::version_manager::VersionInfo;
pub use vault::{Vault, VaultError};

#[cfg(test)]
mod tests;
