use super::*; 
use googletest::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[googletest::test]
fn test_encrypt_file_success() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    let file_path = temp_dir.path().join("test_input.txt");

    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "github - username: foo, password: bar").expect("Failed to write to file");

    let file_path_str = file_path.to_str().expect("Failed to convert file path to string");

    let result = encrypt_file(file_path_str, "foo");

    expect_pred!(result.is_ok());
    let output = result.unwrap();

    expect_pred!(output.ends_with(".enc"));
}

#[googletest::test]
fn test_encrypt_file_nonexistent() {
    let result = encrypt_file("non_existent_file.txt", "foo");
    expect_pred!(result.is_err());

    expect_that!(result, err(matches_pattern!(FileError::FileReadError{ .. })));
}

#[googletest::test]
fn test_read_file_contents_file_not_found() {
    let result = read_file_contents("non_existent_file.txt");
    
    expect_pred!(result.is_err());

    expect_that!(result, err(matches_pattern!(FileError::FileReadError { .. })));
}
