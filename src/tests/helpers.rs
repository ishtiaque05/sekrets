use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::encryption::encryptor::encrypt_file;
use crate::secrets::password_generator::prompt_user_password;
use serde_json;

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

    let pass = prompt_user_password();
    encrypt_file(file_path.path().to_str().unwrap(), &pass).expect("Failed to encrypt file")
}

/// Creates an encrypted file with JSONL content from Credential structs.
pub fn make_encrypted_jsonl_file(credentials: &[crate::secrets::credentials::Credential]) -> String {
    let jsonl = credentials
        .iter()
        .map(|c| serde_json::to_string(c).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    make_encrypted_file(&jsonl)
}
