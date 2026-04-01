# Installation Guide

## Prerequisites
Ensure you have Rust installed. If not, install it using:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Running From Source

```sh
git clone https://github.com/ishtiaque05/sekrets.git
cd sekrets
cargo run -- encrypt -f secret.txt
```
Assuming `secret.txt` file exist in current directory

## Updating

Once installed, you can update to the latest version directly:
```sh
sekrets --update
```

If installed to a system path (e.g., via `.deb` package):
```sh
sudo sekrets --update
```

This checks GitHub Releases for the latest version, downloads the binary, and replaces the installed one.
