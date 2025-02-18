# Installation Guide

## Prerequisites
Ensure you have Rust installed. If not, install it using:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Running From Source

```sh
git clone https://github.com/ishitaque05/sekrets.git
cd sekrets
cargo run -- encrypt -f secret.txt
```
Assuming `secret.txt` file exist in current directory


