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

pub fn encrypt_file(filename: &str, password: &str) -> Result<String, FileError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| FileError::HashingError(err.to_string()))?;

    
    let hash_output = password_hash
        .hash
        .ok_or_else(|| FileError::InvalidHashOutput("Hash output is empty".to_string()))?;

   
    let key_bytes = &hash_output.as_bytes()[..32];
    let key = Aes256Gcm::new_from_slice(key_bytes).expect("Failed to create key from hash");

    let nonce_bytes: [u8; 12] = key_bytes[..12]
        .try_into()
        .map_err(|_| FileError::InvalidNonceSize("Invalid nonce size".to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

   
    let mut file = File::open(filename).map_err(|err| FileError::FileReadError(err.to_string()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| FileError::FileReadError(err.to_string()))?;

    
    let tag = key
        .encrypt_in_place_detached(nonce, b"", &mut buffer)
        .map_err(|err| FileError::EncryptionError(err.to_string()))?;

    buffer.extend_from_slice(tag.as_slice());

    
    let encrypted_filename = get_encrypted_file_path(ENCRYPTED_FILENAME).to_string_lossy().into_owned();

    let mut encrypted_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&encrypted_filename)
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;

   
    writeln!(encrypted_file, "{}", salt.as_str())
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;

   
    encrypted_file
        .write_all(&buffer)
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;

    Ok(encrypted_filename)
}

#[cfg(test)]
mod tests;