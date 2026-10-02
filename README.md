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

**Linux (Debian/Ubuntu):** download the GUI `.deb` from the
[latest release](https://github.com/ishtiaque05/sekrets/releases) and install it:

```sh
sudo dpkg -i sekrets-gui_<RELEASE_VERSION>_amd64.deb
sekrets-gui
```

The release tarball (`sekrets-linux.tar.gz`) also contains the prebuilt
`sekrets-gui` binary alongside the `sekrets` CLI.

**macOS:** releases don't currently ship a prebuilt `Sekrets.app`, so build the
bundle yourself and drag it into `/Applications`:

```sh
cargo install cargo-bundle
cargo bundle -p sekrets-gui --release
open target/release/bundle/osx/Sekrets.app
```

**Linux (other distros):** releases don't currently ship a prebuilt
`.AppImage` either — build one yourself with the bundled script. It needs
[`appimagetool`](https://github.com/AppImage/AppImageKit/releases) on your
`PATH` and ImageMagick's `convert` for the icon:

```sh
./scripts/build-appimage.sh
chmod +x target/appimage/sekrets-gui-*.AppImage
./target/appimage/sekrets-gui-*.AppImage
```

### Uninstall

**Linux (`.deb`):**

```sh
sudo apt purge sekrets-gui
```

This only removes the `sekrets-gui` package — it does **not** delete your
`sekrets.enc` vault file, since the GUI's data-cleanup is deliberately left
to the `sekrets` (CLI) package's own uninstall, so a GUI-only purge can't
accidentally delete data you're still using from the CLI.

**macOS:** drag `Sekrets.app` from `/Applications` to the Trash — there's no
package manager involved.

**AppImage:** delete the `sekrets-gui-*.AppImage` file.

### Run from source

If you have the repo checked out and Rust installed, you don't need to
install a package at all — just build and run the GUI crate directly:

```sh
cargo run -p sekrets-gui
```

The first run compiles `sekrets-gui` and its dependencies (a minute or two);
subsequent runs are fast. Use `cargo run -p sekrets-gui --release` for a
faster-running (but slower-to-compile) optimized build. This launches the
same window as the installed app, reading/writing the same `sekrets.enc`
file as any other install method described above.

The GUI reads and writes the same `sekrets.enc` file as the CLI (`~/.local/share/sekrets/encrypted/sekrets.enc`), so both can be used interchangeably on the same machine.

## Uninstallation
- To uninstall run `sudo apt remove sekrets` if installed via `sudo dpkg -i <SEKRETS>.deb`
- You can run `make uninstall` from project dir if you installed it locally using `make install`

##### **1. Install on Ubuntu (Using `.deb` Package)**

You can install Sekrets on Ubuntu using the pre-built `.deb` package.
Download the latest package from [sekrets releases](https://github.com/ishtiaque05/sekrets/releases)

```sh
sudo dpkg -i sekrets_<RELEASE_VERSION>_amd64.deb

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

