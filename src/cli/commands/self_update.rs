use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const GITHUB_OWNER: &str = "ishtiaque05";
const GITHUB_REPO: &str = "sekrets";

pub fn handle_self_update() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: v{}", current_version);
    println!("Checking for updates...");

    let latest = fetch_latest_release()?;

    let latest_version = latest.tag_name.trim_start_matches('v');
    if latest_version == current_version {
        println!("sekrets is up to date (v{}).", current_version);
        return Ok(());
    }

    println!(
        "New version available: v{} (current: v{}). Update? (y/n): ",
        latest_version, current_version
    );
    std::io::stdout().flush()?;

    let response = if std::env::var("TEST_MODE").is_ok() {
        "n".to_string() // Don't actually update in test mode
    } else {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_lowercase()
    };

    if response != "y" && response != "yes" {
        println!("Update cancelled.");
        return Ok(());
    }

    // Find the right asset for this platform
    let asset_name = get_asset_name()?;
    let asset = latest
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            anyhow::anyhow!("No release asset found for this platform: {}", asset_name)
        })?;

    println!("Downloading {}...", asset.name);
    let binary_data = download_asset(&asset.browser_download_url)?;

    // Determine install path
    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Cannot determine binary path: {}", e))?;

    let current_exe = fs::canonicalize(&current_exe).unwrap_or(current_exe);

    // Check if writable
    if !is_path_writable(&current_exe) {
        println!(
            "Permission required. sekrets is installed at {}",
            current_exe.display()
        );
        println!("Run: sudo sekrets --update");
        return Ok(());
    }

    // Extract and replace
    install_update(&binary_data, &current_exe, &asset.name)?;

    println!("Updated to v{}.", latest_version);

    Ok(())
}

#[derive(serde::Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(serde::Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn fetch_latest_release() -> Result<Release> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("sekrets-updater")
        .build()?;

    let response = client.get(&url).send().map_err(|e| {
        anyhow::anyhow!(
            "Failed to check for updates: {}. Check your internet connection.",
            e
        )
    })?;

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(anyhow::anyhow!(
            "GitHub API rate limit exceeded. Try again later."
        ));
    }

    response
        .json::<Release>()
        .map_err(|e| anyhow::anyhow!("Failed to parse release info: {}", e))
}

fn get_asset_name() -> Result<String> {
    if cfg!(target_os = "linux") {
        Ok("sekrets-linux.tar.gz".to_string())
    } else if cfg!(target_os = "macos") {
        Ok("sekrets-macos.tar.gz".to_string())
    } else {
        Err(anyhow::anyhow!(
            "Self-update is not yet supported on this platform. Download manually from GitHub."
        ))
    }
}

fn download_asset(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("sekrets-updater")
        .build()?;

    let response = client.get(url).send()?;
    let bytes = response.bytes()?;
    Ok(bytes.to_vec())
}

fn is_path_writable(path: &std::path::Path) -> bool {
    if let Some(parent) = path.parent() {
        let test_path = parent.join(".sekrets_update_test");
        match fs::write(&test_path, b"test") {
            Ok(_) => {
                let _ = fs::remove_file(&test_path);
                true
            }
            Err(_) => false,
        }
    } else {
        false
    }
}

fn install_update(archive_data: &[u8], target_path: &PathBuf, asset_name: &str) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    if asset_name.ends_with(".tar.gz") {
        let tar_gz = std::io::Cursor::new(archive_data);
        let decoder = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(temp_dir.path())?;

        // Find the sekrets binary in the extracted files
        let binary_path = find_binary_in_dir(temp_dir.path())?;

        // Atomic-ish replace: rename old, copy new, remove old
        let backup_path = target_path.with_extension("old");
        if target_path.exists() {
            fs::rename(target_path, &backup_path)?;
        }
        fs::copy(&binary_path, target_path)?;

        // Set executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(target_path, fs::Permissions::from_mode(0o755))?;
        }

        // Clean up backup
        let _ = fs::remove_file(&backup_path);
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported archive format: {}",
            asset_name
        ));
    }

    Ok(())
}

fn find_binary_in_dir(dir: &std::path::Path) -> Result<PathBuf> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_binary_in_dir(&path) {
                return Ok(found);
            }
        } else if path.file_name().map(|n| n == "sekrets").unwrap_or(false) {
            return Ok(path);
        }
    }
    Err(anyhow::anyhow!("Could not find sekrets binary in archive"))
}
