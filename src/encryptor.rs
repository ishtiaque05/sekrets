use crate::{paths::get_encrypted_file_path, types::FileError};
use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub const ENCRYPTED_FILENAME: &str = "sekrets.enc";

fn derive_encryption_key(password: &str) -> Result<(Aes256Gcm, SaltString, [u8; 12]), FileError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| FileError::HashingError(err.to_string()))?;

    let hash_output = password_hash
        .hash
        .ok_or_else(|| FileError::InvalidHashOutput("Hash output is empty".to_string()))?;

    let key_bytes = &hash_output.as_bytes()[..32];
    let key = Aes256Gcm::new_from_slice(key_bytes)
        .map_err(|_| FileError::KeyGenerationError("Failed to create key from hash".to_string()))?;

    let nonce: [u8; 12] = key_bytes[..12]
        .try_into()
        .map_err(|_| FileError::InvalidNonceSize("Invalid nonce size".to_string()))?;

    Ok((key, salt, nonce))
}

fn read_file_contents(filename: &str) -> Result<Vec<u8>, FileError> {
    let mut file = File::open(filename).map_err(|err| FileError::FileReadError(err.to_string()))?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)
        .map_err(|err| FileError::FileReadError(err.to_string()))?;

    Ok(buffer)
}

fn encrypt_data(
    key: &Aes256Gcm,
    nonce: &[u8; 12],
    data: &mut Vec<u8>,
) -> Result<Vec<u8>, FileError> {
    let nonce = Nonce::from_slice(nonce);

    let tag = key
        .encrypt_in_place_detached(nonce, b"", data)
        .map_err(|err| FileError::EncryptionError(err.to_string()))?;

    data.extend_from_slice(tag.as_slice());
    Ok(data.clone())
}

fn write_encrypted_file(filepath: &str, salt: &SaltString, data: &[u8]) -> Result<(), FileError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filepath)
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;

    writeln!(file, "{}", salt.as_str())
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;

    file.write_all(data)
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;

    Ok(())
}

/// Encrypts a file with the provided password.
///
/// # Arguments
///
/// * `filename` - The path to the file to be encrypted.
/// * `password` - The password used for encryption.
///
/// # Returns
///
/// A `Result<String, FileError>` containing the path to the encrypted file or an error.
///
/// # Example
///
/// ```
/// let result = encrypt_file("my_secret.txt", "strongpassword");
/// ```

pub fn encrypt_file(filename: &str, password: &str) -> Result<String, FileError> {
    let file_contents = read_file_contents(filename)?;

    let encrypted_path = save_encrypted_file(file_contents, password)?;
    Ok(encrypted_path)
}

fn save_encrypted_file(mut data_bytes: Vec<u8>, password: &str) -> Result<String, FileError> {
    let (key, salt, nonce) = derive_encryption_key(password)?;

    let encrypted_data = encrypt_data(&key, &nonce, &mut data_bytes)?;

    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME)
        .to_string_lossy()
        .into_owned();

    write_encrypted_file(&encrypted_filepath, &salt, &encrypted_data)?;

    Ok(encrypted_filepath)
}

pub fn encrypt_text(data: &str, password: &str) -> Result<String, FileError> {
    let data_bytes = data.as_bytes().to_vec();
    let encrypted_path = save_encrypted_file(data_bytes, password)?;

    Ok(encrypted_path)
}

#[cfg(test)]
mod tests;
