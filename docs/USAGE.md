
# Usage Guide for Sekrets

Sekrets provides a command-line interface (CLI) to securely encrypt files and store credentials.

## Encrypt a File
To encrypt a file:
```sh
sekrets encrypt -f secret.txt
```
- The encrypted file is stored as sekrets.enc in the appropriate directory.
- You will be prompted to enter a password for encryption.

## Decrypt Credentials

```sh
sekrets decrypt -a github
```

- You must enter the same password used for encryption
- The credentials for the specified accounts will be displayed

### View Password History

To see the change history for a credential:
```sh
sekrets decrypt -a github -u myusername --history
```

Output:
```
Account: github - Username: myusername, Password: currentpass
  current:  ********     (2026-03-31 10:30 AM EDT)
  v1:       ********     (2026-03-20 02:00 PM EDT)
  v2:       ********     (2026-03-01 09:15 AM EST)
```

- Up to 5 previous password values are tracked per credential.
- Timestamps are displayed in your local timezone.

## Append New Credentials

To add new credentials to the encrypted file:
```sh
sekrets append -a github -u myusername -p mypassword
```

- This securely appends new credentials to the existing encrypted file.

## Copy the Encrypted File
To copy the encrypted file to a new location:

```
sekrets copy -d /backup/location
```
- The encrypted file is copied to the specified directory

## Update Existing Credentials

To update an existing credential:

```
sekrets update -a github -u myusername
```

- You will be prompted to enter a new password for the specified account and username.
- The old password is saved in the credential's history (up to 5 entries).
- The password is securely updated.

## Generate a Secure Password

To generate a secure password:

```
sekrets generate -p
```

- A strong password is generated and displayed.

- This can be used when updating or adding credentials.

## Find Credentials

To find the account name so that you can use the name to decrypt if you ever forgot account name:

```
sekrets find -a foo
```

- Displays the number of matches found with substring `foo` (case insensitive)

- If no account name is found, an empty list [] is returned.

## Export Credentials

To export decrypted secrets to a plaintext file:

```sh
sekrets export -o secrets.txt
```

- You will be prompted for your master password.
- If the output file already exists, you will be asked to confirm overwrite.

## Import an Encrypted File

To import an `.enc` file from another machine or backup:

```sh
sekrets import -f /path/to/backup.enc
```

- You will be prompted for your current master password (if a sekrets file already exists).
- You will be prompted for the import file's password (it may differ).
- The current file is automatically backed up as a version before being replaced.
- If the import file uses the old format, it is automatically migrated to the new format.

## Version Management

Sekrets maintains up to 5 file-level snapshots. Versions are created automatically when you import a file or switch versions.

### List Versions

```sh
sekrets version --list
```

Output:
```
Versions:
  v1  2026-03-01 09:15 AM EST
  v2  2026-03-15 02:00 PM EDT
  v3  2026-03-20 10:30 AM EDT
```

### Switch to a Previous Version

```sh
sekrets version --switch 2
```

- The current file is backed up before switching.
- You will be prompted for the version file's password (it was encrypted with whatever password was active at that time).
- You will be prompted for your current master password to re-encrypt.

## Self-Update

To update sekrets to the latest release:

```sh
sekrets --update
```

- Checks GitHub Releases for a newer version.
- Downloads and replaces the binary.
- If installed to a system path, run with `sudo sekrets --update`.
