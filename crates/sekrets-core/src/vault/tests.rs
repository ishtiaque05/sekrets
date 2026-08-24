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

#[googletest::test]
fn test_add_then_search_finds_by_account_or_username() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    vault.add("github", "alice", "hunter2").expect("add should succeed");

    expect_that!(vault.search("git").len(), eq(1));
    expect_that!(vault.search("alice").len(), eq(1));
    expect_that!(vault.search("nonexistent").len(), eq(0));
    expect_that!(vault.search("").len(), eq(1));
}

#[googletest::test]
fn test_add_duplicate_returns_already_exists() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    vault.add("github", "alice", "hunter2").expect("first add should succeed");

    let result = vault.add("github", "alice", "different");
    expect_that!(
        result,
        err(matches_pattern!(VaultError::AccountAlreadyExists { .. }))
    );
}

#[googletest::test]
fn test_update_missing_returns_not_found() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    let result = vault.update("github", "alice", "new-password");
    expect_that!(
        result,
        err(matches_pattern!(VaultError::AccountWithUsernameNotFound { .. }))
    );
}

#[googletest::test]
fn test_update_existing_changes_password_and_records_history() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    vault.add("github", "alice", "old-password").expect("add should succeed");

    vault.update("github", "alice", "new-password").expect("update should succeed");

    let creds = vault.search("github");
    expect_that!(creds[0].password, eq("new-password"));
    expect_that!(creds[0].history.len(), eq(1));
    expect_that!(creds[0].history[0].password, eq("old-password"));
}

#[googletest::test]
fn test_delete_missing_returns_not_found() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    let result = vault.delete("github", "alice");
    expect_that!(
        result,
        err(matches_pattern!(VaultError::AccountWithUsernameNotFound { .. }))
    );
}

#[googletest::test]
fn test_delete_existing_removes_it() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    vault.add("github", "alice", "hunter2").expect("add should succeed");

    vault.delete("github", "alice").expect("delete should succeed");
    expect_that!(vault.search("github").len(), eq(0));
}

#[googletest::test]
fn test_history_missing_returns_not_found() {
    let vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    let result = vault.history("github", "alice");
    expect_that!(
        result,
        err(matches_pattern!(VaultError::AccountWithUsernameNotFound { .. }))
    );
}

#[googletest::test]
fn test_history_reflects_password_changes() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    vault.add("github", "alice", "v1").expect("add should succeed");
    vault.update("github", "alice", "v2").expect("update should succeed");
    vault.update("github", "alice", "v3").expect("update should succeed");

    let history = vault.history("github", "alice").expect("history should succeed");
    expect_that!(history.len(), eq(2));
    expect_that!(history[0].password, eq("v2"));
    expect_that!(history[1].password, eq("v1"));
}

#[googletest::test]
fn test_change_master_password_then_old_password_fails_to_unlock() {
    let mut vault = Vault::create(TEST_PASSWORD).expect("create should succeed");
    vault.add("github", "alice", "hunter2").expect("add should succeed");

    vault
        .change_master_password("new-master-password")
        .expect("change should succeed");

    let old_unlock = Vault::unlock(TEST_PASSWORD);
    expect_that!(old_unlock, err(matches_pattern!(VaultError::WrongPassword)));

    let new_unlock = Vault::unlock("new-master-password");
    expect_pred!(new_unlock.is_ok());
    expect_that!(new_unlock.unwrap().search("github").len(), eq(1));
}
