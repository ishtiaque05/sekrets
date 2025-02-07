use super::*;
use crate::encryptor::encrypt_file;
use crate::types::FileError;
use tempfile::NamedTempFile;
use std::fs::File;
use std::io::Write;
use googletest::prelude::*;

fn create_temp_plaintext_file(content: &str) -> NamedTempFile {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut file = File::create(temp_file.path()).expect("Failed to open temp file");

    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    file.flush().expect("Failed to flush file");

    temp_file
}

#[googletest::test]
fn test_successful_encryption_and_decryption() {
    let password = "secure_password";
    let original_content = "Hello Rust!";
    
    let temp_file = create_temp_plaintext_file(original_content);

    let encrypted_filename = encrypt_file(temp_file.path().to_str().unwrap(), password)
        .expect("Encryption failed");

    let decrypted_content = decrypt_file(&encrypted_filename, password)
        .expect("Decryption failed");

    expect_that!(decrypted_content, eq(original_content));
}

#[googletest::test]
fn test_decryption_with_wrong_password_fails() {

    let temp_file = create_temp_plaintext_file("Sensitive Data");
    let encrypted_filename = encrypt_file(temp_file.path().to_str().unwrap(), "correct_password")
        .expect("Encryption failed");

    let result = decrypt_file(&encrypted_filename, "wrong pass");
    expect_pred!(result.is_err());
    expect_that!(result, err(matches_pattern!(FileError::EncryptionError(_))));
}

#[googletest::test]
fn test_decryption_of_nonexistent_file_fails() {
    let result = decrypt_file("nonexistent.enc", "password");

    expect_pred!(result.is_err());
    expect_that!(result, err(matches_pattern!(FileError::FileReadError(_))));
}

#[googletest::test]
fn test_decryption_fails_with_invalid_salt() {
    let password = "secure_pass";
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    writeln!(temp_file, "%%%%%%%INVALID_SALT%%%%%%%").expect("Failed to write invalid salt");

    let result = decrypt_file(temp_file.path().to_str().unwrap(), password);

    expect_pred!(result.is_err());
    expect_that!(result, err(matches_pattern!(FileError::InvalidHashOutput(_))));
}
