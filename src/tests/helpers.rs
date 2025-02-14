use std::fs::File;
use std::io::Write;
#[cfg(test)]
use tempfile::NamedTempFile;

pub fn create_temp_plaintext_file(content: &str) -> NamedTempFile {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut file = File::create(temp_file.path()).expect("Failed to open temp file");

    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    file.flush().expect("Failed to flush file");

    temp_file
}
