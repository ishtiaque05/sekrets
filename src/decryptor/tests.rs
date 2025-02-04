use std::fs;

use crate::encryptor::encrypt_file;

use super::*;

#[test]
fn test_encrypt_and_decrypt_file_success() {
    // Setup
    let temp_file_path = "src/fixtures/foo.txt";
    fs::write(temp_file_path, "This is a test file").unwrap();

    let password = "super_secret_password";

    // Encrypt the file
    let result = encrypt_file(temp_file_path, password);
    assert!(result.is_ok());
    let encrypted_file = result.unwrap();
    
   

    // Decrypt the file
    let decrypted_result = decrypt_file(encrypted_file, password);
    assert!(decrypted_result.is_ok());
    let decrypted_content = decrypted_result.unwrap();
    assert_eq!(decrypted_content, "This is a test file");

}