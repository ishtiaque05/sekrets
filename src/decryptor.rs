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

pub fn decrypt_file(filename: &str, password: &str) -> Result<String, FileError> {
    // Open the encrypted file
    let file = File::open(filename).map_err(|err| FileError::FileReadError(err.to_string()))?;
    let mut reader = BufReader::new(file);

    // Read the base64-encoded salt from the beginning of the file
    let mut salt_base64 = String::new();
    reader
        .read_line(&mut salt_base64)
        .map_err(|err| FileError::FileReadError(err.to_string()))?;
    let salt_base64 = salt_base64.trim();

    let salt = SaltString::from_b64(&salt_base64)
        .map_err(|_| FileError::InvalidHashOutput("Invalid salt encoding".to_string()))?;

    // Derive the key from the password and salt
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| FileError::HashingError(err.to_string()))?;

    let hash_output = password_hash
        .hash
        .ok_or_else(|| FileError::InvalidHashOutput("Hash output is empty".to_string()))?;

    let key =
        Aes256Gcm::new_from_slice(hash_output.as_bytes()).expect("Failed to create key from hash");

    let nonce_bytes: [u8; 12] = hash_output.as_bytes()[..12]
        .try_into()
        .map_err(|_| FileError::InvalidNonceSize("Invalid nonce size".to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Read the remaining encrypted data
    let mut encrypted_data = Vec::new();
    reader
        .read_to_end(&mut encrypted_data)
        .map_err(|err| FileError::FileReadError(err.to_string()))?;

    // Separate the ciphertext and the authentication tag
    let encrypted_data_len = encrypted_data.len();
    if encrypted_data_len < 16 {
        return Err(FileError::EncryptionError(
            "Ciphertext too short".to_string(),
        ));
    }
    let tag_start = encrypted_data_len - 16;
    let tag_bytes: [u8; 16] = encrypted_data[tag_start..]
        .try_into()
        .expect("Invalid ciphertext size");

    let ciphertext = &mut encrypted_data[..tag_start];

    // Decrypt the data
    key.decrypt_in_place_detached(nonce, b"", ciphertext, &tag_bytes.into())
        .map_err(|err| FileError::EncryptionError(err.to_string()))?;

    let decrypted_content = String::from_utf8(ciphertext.to_vec())
        .map_err(|err| FileError::EncryptionError(err.to_string()))?;

    Ok(decrypted_content)
}
