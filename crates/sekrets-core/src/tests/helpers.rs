use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::encryption::encryptor::encrypt_file;
use serde_json;

pub const TEST_PASSWORD: &str = "foo";

pub fn create_temp_plaintext_file(content: &str) -> NamedTempFile {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut file = File::create(temp_file.path()).expect("Failed to open temp file");

    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    file.flush().expect("Failed to flush file");

    temp_file
}

pub fn make_encrypted_file(content: &str) -> String {
    let file_path = create_temp_plaintext_file(content);
    encrypt_file(file_path.path().to_str().unwrap(), TEST_PASSWORD).expect("Failed to encrypt file")
}

pub fn make_encrypted_jsonl_file(
    credentials: &[crate::secrets::credentials::Credential],
) -> String {
    let jsonl = credentials
        .iter()
        .map(|c| serde_json::to_string(c).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    make_encrypted_file(&jsonl)
}
