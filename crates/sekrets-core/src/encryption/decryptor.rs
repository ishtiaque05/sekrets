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

    // A salt line that doesn't decode is structural damage to the file, detectable before
    // any key is derived — it can never be caused by typing the wrong master password.
    let salt = SaltString::from_b64(salt_base64.trim()).map_err(|_| {
        FileError::CorruptFile(
            "the salt line is missing or is not valid base64 — the file is damaged, not \
             locked by a different password"
                .to_string(),
        )
    })?;

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
    // Too short to even contain the 16-byte authentication tag: the file was truncated.
    // Also detectable without a key, so it is corruption rather than a wrong password.
    if encrypted_data.len() < 16 {
        return Err(FileError::CorruptFile(format!(
            "the encrypted body is {} bytes, shorter than the 16-byte authentication tag — \
             the file is truncated",
            encrypted_data.len()
        )));
    }

    let tag_start = encrypted_data.len() - 16;
    let tag_bytes: [u8; 16] = encrypted_data[tag_start..]
        .try_into()
        .map_err(|_| FileError::InvalidCiphertext("Invalid tag size".to_string()))?;

    let ciphertext = &mut encrypted_data[..tag_start];

    key.decrypt_in_place_detached(Nonce::from_slice(nonce), b"", ciphertext, &tag_bytes.into())
        .map_err(|err| FileError::EncryptionError(err.to_string()))?;

    // Reached only after the AEAD tag verified, i.e. the key was correct. Non-UTF-8
    // plaintext at this point means the stored contents themselves are damaged.
    String::from_utf8(ciphertext.to_vec()).map_err(|err| {
        FileError::DecryptionError(format!(
            "contents decrypted and authenticated successfully but are not valid UTF-8: {}",
            err
        ))
    })
}

pub fn decrypt_file(filename: &str, password: &str) -> Result<String, FileError> {
    let (salt, mut encrypted_data) = read_encrypted_file(filename)?;
    let (key, nonce) = derive_key_and_nonce(password, &salt)?;
    decrypt_data(&key, &nonce, &mut encrypted_data)
}

#[cfg(test)]
mod tests;
