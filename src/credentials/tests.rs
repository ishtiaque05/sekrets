use super::*;
use crate::tests::helpers::create_temp_plaintext_file;
use crate::{encryptor::encrypt_file, decryptor, paths::get_encrypted_file_path};
use googletest::prelude::*;

#[googletest::test]
fn test_format_as_str() {
    let credential = Credential::new("github".into(), "user123".into(), "pass123".into());
    let expected = "github - username: user123, password: pass123";
    
    expect_that!(credential.format_as_str(), eq(expected));
}

#[googletest::test]
fn test_add_to_encrypted_file() {
    let file_path = create_temp_plaintext_file("bank - username: alice, password: secretpass");

    let password = "testpassword";
    encrypt_file(file_path.path().to_str().unwrap(), password).expect("Encryption should succeed");

    let new_credential = Credential::new("github".into(), "user123".into(), "pass123".into());
    new_credential.add_to_encrypted_file(password).expect("Appending should succeed");

    let encrypted_filepath = get_encrypted_file_path(ENCRYPTED_FILENAME);
    let decrypted_data = decryptor::decrypt_file(&encrypted_filepath.to_string_lossy(), password)
        .expect("Decryption should succeed");

    let expected = "bank - username: alice, password: secretpass\ngithub - username: user123, password: pass123";
    expect_that!(decrypted_data.trim(), eq(expected));
}

#[googletest::test]
fn test_add_to_encrypted_file_fails_if_no_encrypted_file() {

    let new_credential = Credential::new("github".into(), "user123".into(), "pass123".into());
    let result = new_credential.add_to_encrypted_file("testpassword");

    expect_pred!(result.is_err());
    expect_that!(result, err(matches_pattern!(FileError::DoesnotExist(contains_substring(ENCRYPTED_FILENAME)))));
}
