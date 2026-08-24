use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Hashing failed: {0}")]
    HashingError(String),
    #[error("Invalid hash output: {0}")]
    InvalidHashOutput(String),
    #[error("Nonce generation error: {0}")]
    InvalidNonceSize(String),
    #[error("Failed to write to file: {0}")]
    FileWriteError(String),
    #[error("Failed to read to file: {0}")]
    FileReadError(String),
    #[error("Failed to read to file: {0}")]
    EncryptionError(String),
    #[error("Failed to generate key: {0}")]
    KeyGenerationError(String),
    #[error("Invalid Ciphertext: {0}")]
    InvalidCiphertext(String),
    #[error("Decryption Error: {0}")]
    DecryptionError(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum CredentialError {
    #[error("No credentials found for account: `{0}'")]
    AccountNotFound(String),

    #[error("No credentials found for account: `{0}' with username: `{1}'")]
    AccountWithUsernameNotFound(String, String),
}
