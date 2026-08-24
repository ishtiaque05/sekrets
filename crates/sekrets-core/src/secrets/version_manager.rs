use crate::helpers::directories::get_versions_path;
use crate::types::FileError;
use std::fs;
use std::path::PathBuf;

pub const MAX_VERSIONS: usize = 5;

pub struct VersionInfo {
    pub number: usize,
    pub path: PathBuf,
    pub modified: std::time::SystemTime,
}

/// Get the path to a specific version file.
pub fn get_version_file_path(n: usize) -> PathBuf {
    get_versions_path().join(format!("sekrets.v{}.enc", n))
}

/// Snapshot the current encrypted file into the versions directory.
/// Rotates versions: drops v1, shifts v2→v1, ..., saves current as v5 (or next available slot).
pub fn snapshot_current(current_file: &std::path::Path) -> Result<(), FileError> {
    let versions_dir = get_versions_path();

    let next_slot =
        (1..=MAX_VERSIONS).find(|i| !versions_dir.join(format!("sekrets.v{}.enc", i)).exists());

    if let Some(slot) = next_slot {
        fs::copy(current_file, versions_dir.join(format!("sekrets.v{}.enc", slot)))
            .map_err(|e| FileError::FileWriteError(e.to_string()))?;
    } else {
        let v1 = versions_dir.join("sekrets.v1.enc");
        if v1.exists() {
            fs::remove_file(&v1).map_err(|e| FileError::FileWriteError(e.to_string()))?;
        }

        for i in 2..=MAX_VERSIONS {
            let from = versions_dir.join(format!("sekrets.v{}.enc", i));
            let to = versions_dir.join(format!("sekrets.v{}.enc", i - 1));
            if from.exists() {
                fs::rename(&from, &to).map_err(|e| FileError::FileWriteError(e.to_string()))?;
            }
        }

        fs::copy(current_file, versions_dir.join(format!("sekrets.v{}.enc", MAX_VERSIONS)))
            .map_err(|e| FileError::FileWriteError(e.to_string()))?;
    }

    Ok(())
}

/// List all existing versions with their metadata.
pub fn list_versions() -> Result<Vec<VersionInfo>, FileError> {
    let versions_dir = get_versions_path();
    let mut versions = Vec::new();

    for i in 1..=MAX_VERSIONS {
        let path = versions_dir.join(format!("sekrets.v{}.enc", i));
        if path.exists() {
            let metadata = fs::metadata(&path).map_err(|e| FileError::FileReadError(e.to_string()))?;
            let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
            versions.push(VersionInfo { number: i, path, modified });
        }
    }

    versions.sort_by_key(|v| v.number);
    Ok(versions)
}

#[cfg(test)]
mod tests;
