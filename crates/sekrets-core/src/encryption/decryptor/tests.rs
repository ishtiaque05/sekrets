use super::*;
use crate::encryption::encryptor::encrypt_file;
use crate::types::FileError;
use googletest::prelude::*;

use crate::tests::helpers::create_temp_plaintext_file;

#[googletest::test]
fn test_successful_encryption_and_decryption() {
    let password = "secure_password";
    let temp_file = create_temp_plaintext_file("Hello Rust!");

    let encrypted_filename =
        encrypt_file(temp_file.path().to_str().unwrap(), password).expect("Encryption failed");

    println!("Attempting to decrypt: {}", encrypted_filename);

    let decrypted_content = decrypt_file(&encrypted_filename, password).expect("Decryption failed");

    expect_that!(decrypted_content, eq("Hello Rust!"));
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

    let temp_file = create_temp_plaintext_file("%%%%%%%INVALID_SALT%%%%%%%");

    let result = decrypt_file(temp_file.path().to_str().unwrap(), password);

    expect_pred!(result.is_err());
    // A malformed salt line is structural damage, not a wrong password.
    expect_that!(result, err(matches_pattern!(FileError::CorruptFile(_))));
}

#[googletest::test]
fn test_decryption_of_truncated_body_reports_corruption_not_wrong_password() {
    let password = "secure_password";
    let temp_file = create_temp_plaintext_file("Hello Rust!");
    let encrypted_filename =
        encrypt_file(temp_file.path().to_str().unwrap(), password).expect("Encryption failed");

    // Keep the real (valid) salt line, but truncate the body below the 16-byte AEAD tag.
    // The body is binary, so the file is read as bytes and split at the first newline.
    let contents = std::fs::read(&encrypted_filename).expect("read encrypted file");
    let newline = contents
        .iter()
        .position(|b| *b == b'\n')
        .expect("salt line");
    let mut truncated = contents[..=newline].to_vec();
    truncated.extend_from_slice(b"short");
    std::fs::write(&encrypted_filename, truncated).expect("truncate");

    let result = decrypt_file(&encrypted_filename, password);

    expect_that!(result, err(matches_pattern!(FileError::CorruptFile(_))));
}

#[googletest::test]
fn test_decryption_with_wrong_password_is_not_reported_as_corruption() {
    // The AEAD tag mismatch stays a wrong-password signal — the counterpart to the two
    // corruption tests above, so the two conditions can never collapse back together.
    let temp_file = create_temp_plaintext_file("Sensitive Data");
    let encrypted_filename = encrypt_file(temp_file.path().to_str().unwrap(), "correct_password")
        .expect("Encryption failed");

    let result = decrypt_file(&encrypted_filename, "wrong pass");

    expect_that!(result, err(matches_pattern!(FileError::EncryptionError(_))));
}
