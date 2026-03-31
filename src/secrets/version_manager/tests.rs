use super::*;
use crate::helpers::directories::get_versions_path;
use googletest::prelude::*;
use std::fs;

fn write_dummy_version(path: &std::path::Path) {
    fs::write(path, "dummy encrypted content").unwrap();
}

#[googletest::test]
fn test_snapshot_creates_v1_when_no_versions() {
    let versions_dir = get_versions_path();
    let source = versions_dir.parent().unwrap().join("test_source.enc");
    fs::write(&source, "current file content").unwrap();

    snapshot_current(&source).unwrap();

    let v1 = versions_dir.join("sekrets.v1.enc");
    expect_that!(v1.exists(), eq(true));
    expect_that!(fs::read_to_string(&v1).unwrap(), eq("current file content"));
}

#[googletest::test]
fn test_snapshot_rotates_when_full() {
    let versions_dir = get_versions_path();
    let source = versions_dir.parent().unwrap().join("test_source2.enc");

    // Create 5 existing versions
    for i in 1..=MAX_VERSIONS {
        let path = versions_dir.join(format!("sekrets.v{}.enc", i));
        fs::write(&path, format!("content v{}", i)).unwrap();
    }

    fs::write(&source, "new current").unwrap();
    snapshot_current(&source).unwrap();

    // v1 should now contain what was v2
    let v1_content = fs::read_to_string(versions_dir.join("sekrets.v1.enc")).unwrap();
    expect_that!(v1_content, eq("content v2"));

    // v5 should be the new snapshot
    let v5_content = fs::read_to_string(versions_dir.join("sekrets.v5.enc")).unwrap();
    expect_that!(v5_content, eq("new current"));
}

#[googletest::test]
fn test_list_versions_returns_sorted() {
    let versions_dir = get_versions_path();

    for i in 1..=3 {
        let path = versions_dir.join(format!("sekrets.v{}.enc", i));
        fs::write(&path, format!("v{}", i)).unwrap();
    }

    let versions = list_versions().unwrap();
    expect_that!(versions.len(), eq(3));
    expect_that!(versions[0].number, eq(1));
    expect_that!(versions[2].number, eq(3));
}

#[googletest::test]
fn test_list_versions_empty() {
    let versions = list_versions().unwrap();
    expect_that!(versions.len(), eq(0));
}

#[googletest::test]
fn test_get_version_path() {
    let path = get_version_file_path(3);
    assert!(path.to_str().unwrap().contains("sekrets.v3.enc"));
}
