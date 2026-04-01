use anyhow::Result;

use crate::{
    encryption::{decryptor, encryptor},
    helpers::directories::get_encrypted_file_path,
    secrets::version_manager,
};

pub fn handle_version(list: bool, switch: Option<usize>) -> Result<()> {
    if list {
        return list_versions();
    }

    if let Some(n) = switch {
        return switch_version(n);
    }

    Err(anyhow::anyhow!(
        "Use --list to show versions or --switch <n> to restore a version"
    ))
}

fn list_versions() -> Result<()> {
    let versions = version_manager::list_versions()?;

    if versions.is_empty() {
        println!("No versions found.");
        return Ok(());
    }

    println!("Versions:");
    for v in &versions {
        let time_str = {
            use chrono::{DateTime, Local};
            let datetime: DateTime<Local> = v.modified.into();
            datetime.format("%Y-%m-%d %I:%M %p %Z").to_string()
        };
        println!("  v{}  {}", v.number, time_str);
    }

    Ok(())
}

fn switch_version(n: usize) -> Result<()> {
    let version_path = version_manager::get_version_file_path(n);
    if !version_path.exists() {
        return Err(anyhow::anyhow!("Version v{} does not exist", n));
    }

    // Validate passwords BEFORE any destructive operations
    println!("Enter the password for version v{}:", n);
    let version_password = super::util::prompt_password()?;

    let version_data =
        decryptor::decrypt_file(version_path.to_string_lossy().as_ref(), &version_password)
            .map_err(|_| anyhow::anyhow!("Failed to decrypt version v{}. Wrong password?", n))?;

    println!("Enter your current master password (to re-encrypt):");
    let current_password = super::util::prompt_password()?;

    // Snapshot current AFTER validation succeeds
    let current_path = get_encrypted_file_path(encryptor::ENCRYPTED_FILENAME);
    if current_path.exists() {
        println!("Backing up current file...");
        version_manager::snapshot_current(&current_path)
            .map_err(|e| anyhow::anyhow!("Failed to snapshot current file: {}", e))?;
    }

    // Re-encrypt with current password
    encryptor::encrypt_text(&version_data, &current_password)
        .map_err(|e| anyhow::anyhow!("Failed to re-encrypt: {}", e))?;

    println!("Switched to version v{}.", n);

    Ok(())
}

