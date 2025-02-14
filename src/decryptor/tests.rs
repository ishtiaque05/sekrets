use super::*;
use crate::encryptor::encrypt_file;
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
    expect_that!(
        result,
        err(matches_pattern!(FileError::InvalidHashOutput(_)))
    );
}
