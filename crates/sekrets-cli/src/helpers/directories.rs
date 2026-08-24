use dirs::{config_dir, data_dir};
#[cfg(test)]
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
thread_local! {
    static TEST_TEMP_DIR: RefCell<TempDir> = RefCell::new(TempDir::new()
        .expect("Failed to create a test temp directory"));
}
// static TEST_TEMP_DIR: OnceLock<TempDir> = OnceLock::new();

pub fn get_config_path() -> PathBuf {
    config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("sekrets")
}

pub fn get_data_path() -> PathBuf {
    data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("sekrets")
}

#[cfg(not(test))]
pub fn get_encrypted_file_path(file_name: &str) -> PathBuf {
    if let Ok(test_dir) = std::env::var("SEKRETS_TEST_DIR") {
        let temp_dir = PathBuf::from(test_dir);
        fs::create_dir_all(&temp_dir).expect("Failed to create test temp directory");
        temp_dir.join(file_name)
    } else {
        let mut path = get_data_path();
        path.push("encrypted");
        fs::create_dir_all(&path).expect("Failed to create encrypted files directory");
        path.push(file_name);
        path
    }
}

#[cfg(test)]
fn get_test_temp_dir() -> PathBuf {
    TEST_TEMP_DIR.with(|temp_dir| temp_dir.borrow().path().to_path_buf())
}

#[cfg(test)]
pub fn get_encrypted_file_path(file_name: &str) -> PathBuf {
    let temp_dir = get_test_temp_dir();
    let encrypted_dir = temp_dir.join("encrypted");

    fs::create_dir_all(&encrypted_dir).expect("Failed to create encrypted directory");

    encrypted_dir.join(file_name)
}

#[cfg(not(test))]
pub fn get_versions_path() -> PathBuf {
    if let Ok(test_dir) = std::env::var("SEKRETS_TEST_DIR") {
        let versions_dir = PathBuf::from(test_dir).join("versions");
        fs::create_dir_all(&versions_dir).expect("Failed to create test versions directory");
        versions_dir
    } else {
        let mut path = get_data_path();
        path.push("versions");
        fs::create_dir_all(&path).expect("Failed to create versions directory");
        path
    }
}

#[cfg(test)]
pub fn get_versions_path() -> PathBuf {
    let temp_dir = get_test_temp_dir();
    let versions_dir = temp_dir.join("versions");
    fs::create_dir_all(&versions_dir).expect("Failed to create versions directory");
    versions_dir
}

pub fn ensure_dirs() {
    for path in &[get_config_path(), get_data_path()] {
        if !path.exists() {
            fs::create_dir_all(path).expect("Failed to create directory");
        }
    }
}

#[cfg(test)]
mod tests;
