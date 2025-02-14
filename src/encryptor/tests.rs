use super::*;
use crate::tests::helpers::create_temp_plaintext_file;
use googletest::prelude::*;

#[googletest::test]
fn test_encrypt_file_success() {
    let result = encrypt_file(
        create_temp_plaintext_file("foo").path().to_str().unwrap(),
        "foo",
    );

    expect_pred!(result.is_ok());
    let output = result.unwrap();

    expect_pred!(output.ends_with(".enc"));
}

#[googletest::test]
fn test_encrypt_file_nonexistent() {
    let result = encrypt_file("non_existent_file.txt", "foo");
    expect_pred!(result.is_err());

    expect_that!(
        result,
        err(matches_pattern!(FileError::FileReadError { .. }))
    );
}

#[googletest::test]
fn test_read_file_contents_file_not_found() {
    let result = read_file_contents("non_existent_file.txt");

    expect_pred!(result.is_err());

    expect_that!(
        result,
        err(matches_pattern!(FileError::FileReadError { .. }))
    );
}
