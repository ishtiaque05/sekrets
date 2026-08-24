use super::*;
use crate::tests::helpers::TEST_PASSWORD;
use googletest::prelude::*;

#[googletest::test]
fn test_locate_when_absent() {
    expect_that!(Vault::locate(), none());
}

#[googletest::test]
fn test_create_then_locate_then_unlock_roundtrip() {
    let created = Vault::create(TEST_PASSWORD);
    expect_pred!(created.is_ok());

    expect_that!(Vault::locate(), some(anything()));

    let unlocked = Vault::unlock(TEST_PASSWORD);
    expect_pred!(unlocked.is_ok());
}

#[googletest::test]
fn test_unlock_missing_file_returns_file_not_found() {
    let result = Vault::unlock(TEST_PASSWORD);
    expect_that!(result, err(matches_pattern!(VaultError::FileNotFound(_))));
}

#[googletest::test]
fn test_unlock_wrong_password_returns_wrong_password() {
    Vault::create(TEST_PASSWORD).expect("create should succeed");

    let result = Vault::unlock("definitely-not-the-password");
    expect_that!(result, err(matches_pattern!(VaultError::WrongPassword)));
}

#[googletest::test]
fn test_needs_migration_false_for_freshly_created_vault() {
    let vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    expect_that!(vault.needs_migration(), eq(false));
}

#[googletest::test]
fn test_migrate_legacy_format() {
    use crate::tests::helpers::make_encrypted_file;
    let _ = make_encrypted_file("github - username: foo, password: bar");

    let mut vault = Vault::unlock(TEST_PASSWORD).expect("unlock should succeed");
    expect_that!(vault.needs_migration(), eq(true));

    let migrate_result = vault.migrate();
    expect_pred!(migrate_result.is_ok());
    expect_that!(vault.needs_migration(), eq(false));
}
