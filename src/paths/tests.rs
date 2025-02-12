use super::*; 
use googletest::prelude::*;
use std::path::PathBuf;

#[googletest::test]
fn test_get_config_path() {
    let config_path = get_config_path();

    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    let expected_path = &config_dir.join("sekrets");

    expect_that!(config_path, eq(expected_path));
}

#[googletest::test]
fn test_get_data_path() {
    let data_path = get_data_path();

    let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
    let expected_path = &data_dir.join("sekrets");

    expect_that!(data_path, eq(expected_path));
}
