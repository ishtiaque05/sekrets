use super::*;
use googletest::prelude::*;
use std::path::PathBuf;

#[googletest::test]
fn test_get_config_path() {
    let config_path = get_config_path();

    let config_dir = resolve_base_dir(dirs::config_dir(), dirs::home_dir(), ".config");
    let expected_path = &config_dir.join("sekrets");

    expect_that!(config_path, eq(expected_path));
}

#[googletest::test]
fn test_get_data_path() {
    let data_path = get_data_path();

    let data_dir = resolve_base_dir(dirs::data_dir(), dirs::home_dir(), ".local/share");
    let expected_path = &data_dir.join("sekrets");

    expect_that!(data_path, eq(expected_path));
}

#[test]
fn test_get_versions_path_exists() {
    let path = get_versions_path();
    assert!(path.to_str().unwrap().contains("versions"));
}

#[googletest::test]
fn test_resolve_base_dir_prefers_platform_dir() {
    let resolved = resolve_base_dir(
        Some(PathBuf::from("/platform/share")),
        Some(PathBuf::from("/home/someone")),
        ".local/share",
    );
    expect_that!(resolved, eq(&PathBuf::from("/platform/share")));
}

#[googletest::test]
fn test_resolve_base_dir_falls_back_to_home() {
    let resolved = resolve_base_dir(None, Some(PathBuf::from("/home/someone")), ".local/share");
    expect_that!(resolved, eq(&PathBuf::from("/home/someone/.local/share")));
}

#[googletest::test]
fn test_resolve_base_dir_without_home_never_yields_a_literal_tilde_component() {
    // Regression test: the old fallback was `PathBuf::from("~/.local/share")`, which is
    // never expanded outside a shell and produced a directory literally named `~`.
    let resolved = resolve_base_dir(None, None, ".local/share");
    expect_that!(resolved, eq(&PathBuf::from(".local/share")));
    expect_that!(
        resolved.components().any(|c| c.as_os_str() == "~"),
        eq(false)
    );
}

#[googletest::test]
fn test_ensure_dir_creates_missing_directories() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let nested = temp.path().join("a").join("b").join("c");

    expect_pred!(ensure_dir(&nested).is_ok());
    expect_that!(nested.is_dir(), eq(true));
}

#[googletest::test]
fn test_ensure_dir_returns_error_instead_of_panicking_when_blocked_by_a_file() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let blocker = temp.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").expect("write blocker");

    let result = ensure_dir(&blocker.join("encrypted"));

    expect_that!(result, err(matches_pattern!(FileError::FileWriteError(_))));
}

#[googletest::test]
fn test_ensure_parent_dir_creates_the_parent_of_a_file_path() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let file = temp.path().join("encrypted").join("sekrets.enc");

    expect_pred!(ensure_parent_dir(&file).is_ok());
    expect_that!(file.parent().unwrap().is_dir(), eq(true));
}

#[googletest::test]
fn test_ensure_parent_dir_is_a_noop_for_a_bare_file_name() {
    expect_pred!(ensure_parent_dir(Path::new("sekrets.enc")).is_ok());
}

#[googletest::test]
fn test_ensure_dirs_at_reports_io_failure_rather_than_panicking() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let blocker = temp.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").expect("write blocker");

    let result = ensure_dirs_at(&[temp.path().join("ok"), blocker.join("nope")]);

    expect_that!(result, err(matches_pattern!(FileError::FileWriteError(_))));
}

#[googletest::test]
fn test_path_queries_never_create_directories() {
    // `get_encrypted_file_path` / `get_versions_path` are pure lookups: `Vault::locate()`
    // must not have the side effect of creating the data directory. Comparing existence
    // before and after keeps this deterministic regardless of what earlier tests in the
    // shared thread-local temp dir already created.
    let encrypted_parent_before = get_encrypted_file_path("probe.enc")
        .parent()
        .map(|p| p.exists());
    let versions_before = get_versions_path().exists();

    let encrypted = get_encrypted_file_path("probe.enc");
    let versions = get_versions_path();

    expect_that!(
        encrypted.parent().map(|p| p.exists()),
        eq(encrypted_parent_before)
    );
    expect_that!(versions.exists(), eq(versions_before));
    expect_that!(encrypted.exists(), eq(false));
}
