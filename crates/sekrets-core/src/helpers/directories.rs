use dirs::{config_dir, data_dir};
#[cfg(any(test, feature = "test-utils"))]
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

#[cfg(any(test, feature = "test-utils"))]
use tempfile::TempDir;

#[cfg(any(test, feature = "test-utils"))]
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

#[cfg(any(test, feature = "test-utils"))]
fn get_test_temp_dir() -> PathBuf {
    TEST_TEMP_DIR.with(|temp_dir| temp_dir.borrow().path().to_path_buf())
}

// `SEKRETS_TEST_DIR` is checked unconditionally (in both the production and the
// test/test-utils build) so that a compiled `sekrets` binary always honors it when set,
// regardless of which feature set it happened to be built with. This matters because Cargo
// unifies a package's normal- and dev-dependency features into a single build within one
// `cargo test` invocation: the `sekrets` bin (a normal, non-test target of `sekrets-cli`) can
// end up linked against a `sekrets-core` compiled with `test-utils` active simply because the
// crate's own test targets require it in that same session. The integration tests in
// `crates/sekrets-cli/tests/cli_tests.rs` always set `SEKRETS_TEST_DIR` before invoking the
// binary, so checking it first keeps them correctly isolated no matter which way the bin was
// built, while in-process unit tests (which don't set it) still fall through to the
// thread-local isolated temp dir below.
pub fn get_encrypted_file_path(file_name: &str) -> PathBuf {
    if let Ok(test_dir) = std::env::var("SEKRETS_TEST_DIR") {
        let temp_dir = PathBuf::from(test_dir);
        fs::create_dir_all(&temp_dir).expect("Failed to create test temp directory");
        return temp_dir.join(file_name);
    }

    get_encrypted_file_path_default(file_name)
}

#[cfg(not(any(test, feature = "test-utils")))]
fn get_encrypted_file_path_default(file_name: &str) -> PathBuf {
    let mut path = get_data_path();
    path.push("encrypted");
    fs::create_dir_all(&path).expect("Failed to create encrypted files directory");
    path.push(file_name);
    path
}

#[cfg(any(test, feature = "test-utils"))]
fn get_encrypted_file_path_default(file_name: &str) -> PathBuf {
    let temp_dir = get_test_temp_dir();
    let encrypted_dir = temp_dir.join("encrypted");

    fs::create_dir_all(&encrypted_dir).expect("Failed to create encrypted directory");

    encrypted_dir.join(file_name)
}

pub fn get_versions_path() -> PathBuf {
    if let Ok(test_dir) = std::env::var("SEKRETS_TEST_DIR") {
        let versions_dir = PathBuf::from(test_dir).join("versions");
        fs::create_dir_all(&versions_dir).expect("Failed to create test versions directory");
        return versions_dir;
    }

    get_versions_path_default()
}

#[cfg(not(any(test, feature = "test-utils")))]
fn get_versions_path_default() -> PathBuf {
    let mut path = get_data_path();
    path.push("versions");
    fs::create_dir_all(&path).expect("Failed to create versions directory");
    path
}

#[cfg(any(test, feature = "test-utils"))]
fn get_versions_path_default() -> PathBuf {
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
