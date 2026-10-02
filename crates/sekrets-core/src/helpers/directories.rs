use dirs::{config_dir, data_dir, home_dir};
#[cfg(any(test, feature = "test-utils"))]
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(any(test, feature = "test-utils"))]
use tempfile::TempDir;

use crate::types::FileError;

#[cfg(any(test, feature = "test-utils"))]
thread_local! {
    static TEST_TEMP_DIR: RefCell<TempDir> = RefCell::new(TempDir::new()
        .expect("Failed to create a test temp directory"));
}
// static TEST_TEMP_DIR: OnceLock<TempDir> = OnceLock::new();

/// Resolves a base directory, preferring the platform-specific location, then
/// `$HOME/<relative>`, then `<relative>` under the process working directory.
///
/// Split out from [`get_config_path`]/[`get_data_path`] so the fallback chain is unit
/// testable without manipulating the environment. The previous fallback was a literal
/// `PathBuf::from("~/.config")`: nothing expands `~` outside a shell, so when
/// `dirs::config_dir()` returned `None` the app silently created — and wrote the vault
/// into — a directory named literally `~` inside whatever directory it started in.
fn resolve_base_dir(
    platform_dir: Option<PathBuf>,
    home: Option<PathBuf>,
    relative: &str,
) -> PathBuf {
    platform_dir
        .or_else(|| home.map(|h| h.join(relative)))
        .unwrap_or_else(|| PathBuf::from(relative))
}

pub fn get_config_path() -> PathBuf {
    resolve_base_dir(config_dir(), home_dir(), ".config").join("sekrets")
}

pub fn get_data_path() -> PathBuf {
    resolve_base_dir(data_dir(), home_dir(), ".local/share").join("sekrets")
}

#[cfg(any(test, feature = "test-utils"))]
fn get_test_temp_dir() -> PathBuf {
    TEST_TEMP_DIR.with(|temp_dir| temp_dir.borrow().path().to_path_buf())
}

/// Creates `path` and any missing parents, reporting failure as a typed [`FileError`]
/// rather than panicking. `sekrets-core` must never panic on fallible I/O: these calls
/// are reachable from the GUI's very first action, where a panic inside an async task
/// leaves the user with an aborted process or a permanently blank window.
pub fn ensure_dir(path: &Path) -> Result<(), FileError> {
    fs::create_dir_all(path).map_err(|err| {
        FileError::FileWriteError(format!(
            "could not create directory {}: {}",
            path.display(),
            err
        ))
    })
}

/// Creates the parent directory of `path`, if it has a non-empty one.
pub fn ensure_parent_dir(path: &Path) -> Result<(), FileError> {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => ensure_dir(parent),
        _ => Ok(()),
    }
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
//
// This is a *pure path query*: it never creates directories. Creating them here made
// `Vault::locate()` — a read-only lookup — silently mkdir the data directory, and forced
// the creation to happen where no `Result` could be returned (hence the old `.expect()`
// panics). Directory creation now lives in the write paths that already return a
// `Result`: `encryptor::write_encrypted_file` and `version_manager::snapshot_current`.
pub fn get_encrypted_file_path(file_name: &str) -> PathBuf {
    if let Ok(test_dir) = std::env::var("SEKRETS_TEST_DIR") {
        return PathBuf::from(test_dir).join(file_name);
    }

    get_encrypted_file_path_default(file_name)
}

#[cfg(not(any(test, feature = "test-utils")))]
fn get_encrypted_file_path_default(file_name: &str) -> PathBuf {
    get_data_path().join("encrypted").join(file_name)
}

#[cfg(any(test, feature = "test-utils"))]
fn get_encrypted_file_path_default(file_name: &str) -> PathBuf {
    get_test_temp_dir().join("encrypted").join(file_name)
}

/// Pure path query, like [`get_encrypted_file_path`] — the versions directory is created
/// by `version_manager::snapshot_current`, the only writer.
pub fn get_versions_path() -> PathBuf {
    if let Ok(test_dir) = std::env::var("SEKRETS_TEST_DIR") {
        return PathBuf::from(test_dir).join("versions");
    }

    get_versions_path_default()
}

#[cfg(not(any(test, feature = "test-utils")))]
fn get_versions_path_default() -> PathBuf {
    get_data_path().join("versions")
}

#[cfg(any(test, feature = "test-utils"))]
fn get_versions_path_default() -> PathBuf {
    get_test_temp_dir().join("versions")
}

/// Creates the config and data directories up front (used by the CLI at startup).
pub fn ensure_dirs() -> Result<(), FileError> {
    ensure_dirs_at(&[get_config_path(), get_data_path()])
}

fn ensure_dirs_at(paths: &[PathBuf]) -> Result<(), FileError> {
    for path in paths {
        ensure_dir(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
