use clap::{Command, Arg, ArgAction};
use anyhow::Result;  // Import Result from anyhow for error handling

use rand::rngs::OsRng; 
use thiserror::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use aes_gcm::{aead::{AeadInPlace, KeyInit},Aes256Gcm, Nonce};
use argon2::{
    password_hash::{SaltString, PasswordHasher},
    Argon2,
};

#[derive(Error, Debug)]
pub enum FileError {
    // ... (existing error variants)
    #[error("Hashing failed: {0}")]
    HashingError(String),
    #[error("Invalid hash output: {0}")]
    InvalidHashOutput(String),
    #[error("Nonce generation error: {0}")]
    InvalidNonceSize(String),
    #[error("Failed to write to file: {0}")]
    FileWriteError(String),
    #[error("Failed to read to file: {0}")]
    FileReadError(String),
    #[error("Failed to read to file: {0}")]
    EncryptionError(String),
}

fn main() -> Result<()> {
    let matches = Command::new("My CLI App")
        .version("1.0")
        .author("Syed Ishtiaque Ahmad")
        .about("Sssshhh its a secret!!")
        .arg(Arg::new("encrypt")
             .long("encrypt")
             .value_name("FILE")
             .help("Encrypts the specified file")
             .action(ArgAction::Set))
        .arg(Arg::new("sekrets")
            .long("sekrets")
            .help("Decrypts the encrypted file to reveal passwords"))
       .arg(Arg::new("keyword")
            .help("The keyword for which to retrieve the password, e.g., --github")
            .long_help("Specific keyword like --github to fetch the password for GitHub")
            .action(ArgAction::Append))  // Changed to Append to allow multiple values
        .get_matches();

    if let Some(file) = matches.get_one::<String>("encrypt") {
        println!("File to encrypt: {}", file);
        println!("Encrypted filed {:?}", encrypt_file(file, "foo"))
    }

    if matches.contains_id("sekrets") {
        println!("Sekrets option selected");
    }

    if let Some(keywords) = matches.get_many::<String>("keyword") {
        for keyword in keywords {
            println!("Keyword: {}", keyword);
        }
    }
       

    Ok(())
}

pub fn encrypt_file(filename: &str, password: &str) -> Result<String, FileError> {
    let salt = SaltString::generate(&mut OsRng);

    println!("Filename {0}", filename);
    // Hash the password with Argon2
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| FileError::HashingError(err.to_string()))?;

    // Extract the hash
    let hash_output = password_hash
        .hash
        .ok_or_else(|| FileError::InvalidHashOutput("Hash output is empty".to_string()))?;

    // Create key and nonce
    let key_bytes = &hash_output.as_bytes()[..32];
    let key = Aes256Gcm::new_from_slice(key_bytes)
        .expect("Failed to create key from hash"); 

    let nonce_bytes: [u8; 12] = hash_output.as_bytes()[..12]
        .try_into()
        .map_err(|_| FileError::InvalidNonceSize("Invalid nonce size".to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Read file contents
    let mut file = File::open(filename).map_err(|err| FileError::FileReadError(err.to_string()))?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)
        .map_err(|err| FileError::FileReadError(err.to_string()))?;

    // Encrypt in-place
    let tag = key.encrypt_in_place_detached(nonce, b"", &mut buffer)
        .map_err(|err| FileError::EncryptionError(err.to_string()))?;
    
    buffer.extend_from_slice(tag.as_slice());

    // Write the encrypted data
    let encrypted_filename = format!("{}.enc", filename);
    let mut encrypted_file = OpenOptions::new()
        .write(true)
        .create(true) 
        .open(&encrypted_filename)
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;
    encrypted_file.write_all(&buffer)
        .map_err(|err| FileError::FileWriteError(err.to_string()))?;

    // Encode salt and create output string
    let encoded_salt = base64::encode(salt.as_str());
    let output = format!("{}\n{}", encrypted_filename, encoded_salt);

    Ok(output)
}