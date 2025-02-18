
# Usage Guide for Sekrets

Sekrets provides a command-line interface (CLI) to securely encrypt files and store credentials.

## Encrypt a File
To encrypt a file:
```sh
sekret encrypt -f secret.txt
```
- The encrypted file is stored as sekrets.enc in the appropriate directory.
- You will be prompted to enter a password for encryption.

## Decrypt Credentials

```sh
sekret decrypt -a github
```

- You must enter the same password used for encryption
- The credentials for the specified accounts will be displayed

## Append New Credentials

To add new credentials to the encrypted file:
```sh
sekret append -a github -u myusername -p mypassword
```

- This securely appends new credentials to the existing encrypted file.

## Copy the Encrypted File
To copy the encrypted file to a new location:

```
sekret copy -d /backup/location
```
- The encrypted file is copied to the specified directory
