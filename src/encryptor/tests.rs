// src/tests/encrypt_file_tests.rs

use super::*;
use std::fs; // Added for file manipulation

#[test]
fn test_encrypt_file_success() {
    // Setup
    let temp_file_path = "test_file.txt";
    fs::write(temp_file_path, "This is a test file").unwrap();

    let password = "super_secret_password";

    // Exercise
    let result = encrypt_file(temp_file_path, password);

    // Verify
    assert!(result.is_ok());
    let output = result.unwrap();

    assert!(output.ends_with(".enc")); // Check encrypted file extension
    
    // Cleanup
    fs::remove_file(temp_file_path).unwrap();
    fs::remove_file(parts[0]).unwrap(); 
}


// Add more tests to cover different scenarios:
// - Incorrect password
// - File doesn't exist
// - Empty file
// ...
