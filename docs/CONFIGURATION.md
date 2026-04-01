# Configuration Guide

Sekrets stores encrypted files in a standard directory structure.

## File Paths
- **Configuration directory:** `~/.config/sekrets/`
- **Encrypted data storage:** `~/.local/share/sekrets/encrypted/`
- **File-level version snapshots:** `~/.local/share/sekrets/versions/`

## Changing Storage Location
By default, encrypted files are stored in:
```sh
~/.local/share/sekrets/encrypted/sekrets.enc
```

## Versions Directory

When you import a file or switch versions, the current `sekrets.enc` is saved as a snapshot:
```
~/.local/share/sekrets/versions/
  ├── sekrets.v1.enc   # oldest
  ├── sekrets.v2.enc
  ├── sekrets.v3.enc
  ├── sekrets.v4.enc
  └── sekrets.v5.enc   # newest
```

Up to 5 versions are kept. When a new version is created and the limit is reached, the oldest is automatically deleted.

## Data Format

Sekrets stores credentials internally in JSONL format (one JSON object per line) inside the encrypted file. Each credential includes:
- Account name
- Username
- Password
- Timestamp (UTC) of when it was last changed
- Up to 5 historical password values with timestamps

### Migration from Older Versions

If you upgrade from an older version of sekrets that used the flat-text format, sekrets will automatically detect this on first use and prompt you to migrate:

```
Your sekrets file uses an older format. It will be upgraded to the new format.
A backup of your current file will be saved before migrating.
Proceed? (y/n):
```

A backup of your old file is saved as `versions/sekrets.v1.enc` before migration. The migration is a one-time operation.
