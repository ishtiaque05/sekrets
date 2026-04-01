use anyhow::Result;
use std::path::Path;

use crate::{
    encryption::{decryptor, encryptor},
    helpers::directories::get_encrypted_file_path,
    secrets::{credential_file_parser::CredentialFileParser, version_manager},
};

pub fn handle_import(file: &str) -> Result<()> {
    let import_path = Path::new(file);
    if !import_path.exists() {
        return Err(anyhow::anyhow!("Import file not found: {}", file));
    }

    let current_enc_path = get_encrypted_file_path(encryptor::ENCRYPTED_FILENAME);
    let has_existing = current_enc_path.exists();

    // Prompt for current master password (if existing file)
    let current_password = if has_existing {
        println!("Enter your current master password:");
        let pass = prompt_password()?;

        // Validate by attempting to decrypt
        let current_path_str = current_enc_path.to_string_lossy().to_string();
        decryptor::decrypt_file(&current_path_str, &pass)
            .map_err(|_| anyhow::anyhow!("Failed to decrypt current file. Wrong password?"))?;

        Some(pass)
    } else {
        None
    };

    // Prompt for import file password
    println!("Enter the password for the import file:");
    let import_password = prompt_password()?;

    // Decrypt and validate import file
    let import_data = decryptor::decrypt_file(file, &import_password)
        .map_err(|_| anyhow::anyhow!("Failed to decrypt import file. Wrong password?"))?;

    // Validate it's a valid sekrets format
    let parser = CredentialFileParser::new(import_data.clone());
    let credentials = parser.get_all_credentials();
    if credentials.is_empty() && !import_data.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Import file doesn't contain valid sekrets data"
        ));
    }

    // Snapshot current file before replacing (if it exists)
    if has_existing {
        version_manager::snapshot_current(&current_enc_path)
            .map_err(|e| anyhow::anyhow!("Failed to create version snapshot: {}", e))?;
        println!("Current file backed up to versions directory.");
    }

    // If legacy format, migrate to JSONL
    let final_data = if parser.is_legacy_format() {
        use chrono::Utc;
        let mut creds = credentials;
        let now = Utc::now().to_rfc3339();
        for cred in creds.values_mut() {
            if cred.ts.is_empty() {
                cred.ts = now.clone();
            }
        }
        CredentialFileParser::serialize_to_jsonl(&creds)
    } else {
        import_data
    };

    // Re-encrypt with current master password (or import password if no existing file)
    let encrypt_password = current_password.unwrap_or(import_password);
    encryptor::encrypt_text(&final_data, &encrypt_password)
        .map_err(|e| anyhow::anyhow!("Failed to encrypt imported data: {}", e))?;

    println!("Import successful!");

    Ok(())
}

fn prompt_password() -> Result<String> {
    if std::env::var("TEST_MODE").is_ok() || cfg!(test) {
        Ok(std::env::var("USER_TEST_PASS").unwrap_or_else(|_| "foo".to_string()))
    } else {
        use rpassword::read_password;
        use std::io::Write;
        std::io::stdout().flush()?;
        Ok(read_password().expect("Failed to read password"))
    }
}
