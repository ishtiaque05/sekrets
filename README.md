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
- 📥 Import `.enc` files from other machines or backups.
- 🕒 Track password change history (up to 5 per credential).
- 📦 File-level versioning with up to 5 snapshots.
- ⬆️ Self-update from GitHub Releases.

# Security Considerations

Sekrets uses **AES-256-GCM** encryption and **Argon2 password hashing** to ensure security.

## Encryption Details
- Uses **AES-256-GCM** for encryption.
- Derives keys using **Argon2** for added security.

## Installation
Rust must be installed to compile and run Sekrets.

## GUI

Sekrets also ships a desktop GUI (`sekrets-gui`) that finds your existing
`sekrets.enc` file and lets you unlock, search, view, edit, and manage
credentials without the command line.

### Install

**macOS:** download `Sekrets.app` from the latest release and drag it into
`/Applications`, or build it yourself:

```sh
cargo install cargo-bundle
cargo bundle -p sekrets-gui --release
open target/release/bundle/osx/Sekrets.app
```

**Linux (Debian/Ubuntu):**

```sh
sudo dpkg -i sekrets-gui_<RELEASE_VERSION>_amd64.deb
sekrets-gui
```

**Linux (other distros):** download the `.AppImage` from the latest
release, make it executable, and run it:

```sh
chmod +x sekrets-gui-*.AppImage
./sekrets-gui-*.AppImage
```

The GUI reads and writes the same `sekrets.enc` file as the CLI (`~/.local/share/sekrets/encrypted/sekrets.enc`), so both can be used interchangeably on the same machine.

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

## Updating

To update to the latest version:
```sh
sekrets --update
```
If installed to a system path (`/usr/local/bin/`), run with `sudo`:
```sh
sudo sekrets --update
```

## Uninstallation

##### Ubuntu
```
sudo apt purge sekrets
```

### Versioning

We rely on https://semver.org/ for this project.

