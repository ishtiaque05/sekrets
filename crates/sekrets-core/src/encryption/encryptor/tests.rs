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
fn test_write_encrypted_file_creates_missing_parent_directory() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let target = temp.path().join("encrypted").join("sekrets.enc");
    let salt = SaltString::generate(&mut OsRng);

    let result = write_encrypted_file(target.to_str().unwrap(), &salt, b"ciphertext");

    expect_pred!(result.is_ok());
    expect_that!(target.is_file(), eq(true));
}

#[googletest::test]
fn test_write_encrypted_file_reports_error_instead_of_panicking_when_dir_is_blocked() {
    // A plain file sitting where the directory should be: `create_dir_all` fails. This is
    // exactly the case that used to `.expect()`-panic inside `get_encrypted_file_path`,
    // which for the GUI meant a panic on a tokio worker thread during startup.
    let temp = tempfile::TempDir::new().expect("temp dir");
    let blocker = temp.path().join("encrypted");
    std::fs::write(&blocker, b"not a directory").expect("write blocker");
    let salt = SaltString::generate(&mut OsRng);

    let result = write_encrypted_file(
        blocker.join("sekrets.enc").to_str().unwrap(),
        &salt,
        b"ciphertext",
    );

    expect_that!(result, err(matches_pattern!(FileError::FileWriteError(_))));
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
