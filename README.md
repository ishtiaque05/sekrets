# Sekrets - Secure File Encryption in Rust
![Build](https://github.com/ishtiaque05/sekrets/actions/workflows/rust.yml/badge.svg)

Sekrets is a Rust-based file encryption tool that securely encrypts and stores sensitive credentials. It uses **AES-256-GCM encryption** with **Argon2 password hashing** for strong key derivation.

## Features
For detailed usage instructions, refer to the [Usage Guide](./docs/USAGE.md).
- 📂 Encrypt and decrypt files securely.
- 🔑 Store credentials safely using strong encryption.
- 🔄 Append new credentials to an existing encrypted file.
- 🔄 Update existing credentials securely.
- 📋 Copy encrypted files to a new location.
- 🔢 Generate strong passwords for credentials.
- 🔍 Find account name.

# Security Considerations

Sekrets uses **AES-256-GCM** encryption and **Argon2 password hashing** to ensure security.

## Encryption Details
- Uses **AES-256-GCM** for encryption.
- Derives keys using **Argon2** for added security.

## Installation
Rust must be installed to compile and run Sekrets.

## Uninstallation
- To uninstall run `sudo apt remove sekrets` if installed via `sudo dpkg -i <SEKRETS>.deb`
- You can run `make uninstall` from project dir if you installed it locally using `make install`

##### **1. Install on Ubuntu (Using `.deb` Package)**

You can install Sekrets on Ubuntu using the pre-built `.deb` package.
Download the latest package from [sekrets releases](https://github.com/ishtiaque05/sekrets/releases)

```sh
sudo dpkg -i sekrets_<RELEAESE_VERSION>_amd64.deb

sekrets --version # to verify installation
```

More instruction on different ways of installation can be found in the [installation guide](./docs/INSTALLATION.md)

## Uninstallation

##### Ubuntu
```
sudo apt purge sekrets
```

### Versioning

We rely on https://semver.org/ for this project.

