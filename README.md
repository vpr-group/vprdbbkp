# VPR DB Backup - Database Backup & Restore Utility

A robust, efficient tool written in Rust to backup databases directly to different kind cloud storage or local filesystem.

## Features

- **Database Support**: PostgreSQL backup & restore
- **Storage Options**: S3 compatible storage or local filesystem
- **Compression**: Optional compression with configurable levels
- **Flexible Restoration**: Restore from specific backups or automatically use the latest one
- **Listing & Management**: List available backups with filtering options

## Installation

### Quick Installation

#### Linux

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install-cli.sh | sudo bash
```

#### macOS

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install-cli.sh | bash
```

Note: macOS users may need to remove `sudo` depending on their permission settings.

### Install with Dependencies

To also install required system dependencies (PostgreSQL client, gzip):

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install-cli.sh | sudo bash -s -- --with-deps
```

### Static Build Installation (Better Compatibility)

For environments where dynamic linking might be an issue:

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install-cli.sh | sudo bash -s -- --musl
```

### Install from Source (requires Rust toolchain)

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install-cli.sh | sudo bash -s -- --from-source
```

Or manually:

```bash
# Clone the repository
git clone https://github.com/vpr-group/vprs3bkp.git
cd vprs3bkp

# Build the project
cd cli
cargo build --release

# Install the binary
sudo cp target/release/cli /usr/local/bin/vprs3bkp
```

## CLI Usage

The CLI provides a simple interface to backup, restore, and manage database backups.

### Backup a PostgreSQL Database

```bash
# Back up to S3
vprs3bkp backup \
  --database mydb \
  --host db.example.com \
  --username postgres \
  --password mypassword \
  --storage-type s3 \
  --bucket my-backups \
  --region us-east-1 \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAXXXXXXXX \
  --secret-key XXXXXXXXX \
  --prefix backups/postgres

# Back up to local storage
vprs3bkp backup \
  --database mydb \
  --host localhost \
  --username postgres \
  --storage-type local \
  --root /path/to/backups

# Backup with compression (0-9)
vprs3bkp backup \
  --database mydb \
  --compression 6 \
  --storage-type local \
  --root /path/to/backups
```

### List Available Backups

```bash
# List all backups (limited to 10 by default)
vprs3bkp list \
  --storage-type s3 \
  --bucket my-backups \
  --region us-east-1 \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAXXXXXXXX \
  --secret-key XXXXXXXXX

# List backups for a specific database
vprs3bkp list \
  --database mydb \
  --storage-type local \
  --root /path/to/backups

# Show only latest backup
vprs3bkp list \
  --database mydb \
  --latest-only \
  --storage-type local \
  --root /path/to/backups

# List more than the default 10 backups
vprs3bkp list \
  --limit 20 \
  --storage-type s3 \
  --bucket my-backups
```

### Restore a Database

```bash
# Restore from a specific backup file
vprs3bkp restore \
  --database mydb \
  --host localhost \
  --username postgres \
  --filename mydb-2023-10-14T14:30:00.dump \
  --storage-type local \
  --root /path/to/backups

# Restore the most recent backup
vprs3bkp restore \
  --database mydb \
  --host localhost \
  --username postgres \
  --latest \
  --storage-type s3 \
  --bucket my-backups \
  --region us-east-1 \
  --endpoint https://s3.amazonaws.com
```

## Environment Variables

You can use environment variables instead of command-line arguments:

- `PGPASSWORD` - PostgreSQL password
- `S3_BUCKET` - S3 bucket name
- `S3_REGION` - S3 region
- `S3_ENDPOINT` - S3 endpoint URL
- `S3_ACCESS_KEY_ID` or `S3_ACCESS_KEY` - S3 access key
- `S3_SECRET_ACCESS_KEY` or `S3_SECRET_KEY` - S3 secret key

Example:

```bash
export PGPASSWORD=mypassword
export S3_BUCKET=my-backups
export S3_REGION=us-east-1
export S3_ENDPOINT=https://s3.amazonaws.com
export S3_ACCESS_KEY=AKIAXXXXXXXX
export S3_SECRET_KEY=XXXXXXXXX

vprs3bkp backup --database mydb --storage-type s3
```

## Configuration

| Option                    | Description            | Default     |
| ------------------------- | ---------------------- | ----------- |
| **Common Options**        |
| `--source-type`           | Database type          | `postgres`  |
| `--source-name`           | Source identifier      | `default`   |
| `--storage-type`          | Storage backend        | `s3`        |
| `--storage-name`          | Storage identifier     | `default`   |
| `--prefix`                | Path prefix in storage | (none)      |
| **PostgreSQL Options**    |
| `--database`              | Database name          | (required)  |
| `-H, --host`              | Database host          | `localhost` |
| `--port`                  | Database port          | `5432`      |
| `--username`              | Database user          | `postgres`  |
| `--password`              | Database password      | (from env)  |
| **S3 Options**            |
| `--bucket`                | S3 bucket name         | (required)  |
| `--region`                | S3 region              | `us-east-1` |
| `--endpoint`              | S3 endpoint URL        | (required)  |
| `--access-key`            | S3 access key          | (required)  |
| `--secret-key`            | S3 secret key          | (required)  |
| **Local Storage Options** |
| `--root`                  | Local directory path   | (required)  |

## Backup Naming Convention

Backups are automatically named using the format:

```
{source-name}-{database}-{timestamp}.dump
```

For example:

```
default-mydb-2023-10-15T08:45:31Z.dump
```

## Setting Up Cron Jobs

Add to `/etc/cron.d/database-backups`:

```
# Daily PostgreSQL backup at 2:00 AM
0 2 * * * root S3_BUCKET=my-backup-bucket PGPASSWORD=secret /usr/local/bin/vprs3bkp backup --database mydb --username dbuser --host db.example.com --storage-type s3
```

## Prerequisites

- PostgreSQL client tools (`pg_dump` and `psql`) for PostgreSQL backups
- AWS credentials with `s3:PutObject` permissions (for S3 storage)
- Gzip for compression

### Installing Prerequisites on macOS

```bash
# Using Homebrew
brew install postgresql

# Verify installation
psql --version
pg_dump --version
```

### Installing Prerequisites on Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y postgresql-client gzip

# Verify installation
psql --version
pg_dump --version
```

## Troubleshooting

### Common Issues

1. **Connection refused**: Check database host, port, and firewall settings
2. **Access denied**: Verify database credentials
3. **S3 upload failed**: Check S3 credentials and permissions
4. **"Failed to execute psql command: no such file"** or **"pg_dump not found"**: These errors indicate that PostgreSQL client tools are not installed or not in your PATH. Install them with:
   - macOS: `brew install postgresql`
   - Ubuntu/Debian: `sudo apt-get install postgresql-client`

## AWS Authentication

The tool uses the AWS SDK's credential provider chain, which checks for credentials in this order:

1. Command-line arguments (`--access-key`, `--secret-key`)
2. Environment variables (`S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`)
3. AWS shared credentials file (`~/.aws/credentials`)
4. IAM role for Amazon EC2 or ECS task role

## S3-Compatible Storage Providers

To use with S3-compatible storage providers (MinIO, DigitalOcean Spaces, etc.):

```bash
vprs3bkp backup \
  --database mydb \
  --username dbuser \
  --storage-type s3 \
  --bucket my-bucket \
  --endpoint https://minio.example.com \
  --access-key ACCESS_KEY \
  --secret-key SECRET_KEY
```

## License

MIT
