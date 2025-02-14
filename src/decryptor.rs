use crate::types::FileError;
use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use std::io::{BufReader, Read};
use std::{fs::File, io::BufRead};

fn read_encrypted_file(filename: &str) -> Result<(SaltString, Vec<u8>), FileError> {
    let file = File::open(filename).map_err(|err| FileError::FileReadError(err.to_string()))?;
    let mut reader = BufReader::new(file);

    let mut salt_base64 = String::new();
    reader
        .read_line(&mut salt_base64)
        .map_err(|err| FileError::FileReadError(err.to_string()))?;
    let salt = SaltString::from_b64(salt_base64.trim())
        .map_err(|_| FileError::InvalidHashOutput("Invalid salt encoding".to_string()))?;

    let mut encrypted_data = Vec::new();
    reader
        .read_to_end(&mut encrypted_data)
        .map_err(|err| FileError::FileReadError(err.to_string()))?;

    Ok((salt, encrypted_data))
}

fn derive_key_and_nonce(
    password: &str,
    salt: &SaltString,
) -> Result<(Aes256Gcm, [u8; 12]), FileError> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
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

    Ok((key, nonce))
}

fn decrypt_data(
    key: &Aes256Gcm,
    nonce: &[u8; 12],
    encrypted_data: &mut [u8],
) -> Result<String, FileError> {
    if encrypted_data.len() < 16 {
        return Err(FileError::EncryptionError(
            "Ciphertext too short".to_string(),
        ));
    }

    let tag_start = encrypted_data.len() - 16;
    let tag_bytes: [u8; 16] = encrypted_data[tag_start..]
        .try_into()
        .map_err(|_| FileError::InvalidCiphertext("Invalid tag size".to_string()))?;

    let ciphertext = &mut encrypted_data[..tag_start];

    key.decrypt_in_place_detached(Nonce::from_slice(nonce), b"", ciphertext, &tag_bytes.into())
        .map_err(|err| FileError::EncryptionError(err.to_string()))?;

    String::from_utf8(ciphertext.to_vec())
        .map_err(|err| FileError::DecryptionError(err.to_string()))
}

pub fn decrypt_file(filename: &str, password: &str) -> Result<String, FileError> {
    let (salt, mut encrypted_data) = read_encrypted_file(filename)?;
    let (key, nonce) = derive_key_and_nonce(password, &salt)?;
    decrypt_data(&key, &nonce, &mut encrypted_data)
}

#[cfg(test)]
mod tests;
