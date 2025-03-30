# VPR DB Backup - Database Backup & Restore Utility

A simple tool for backing up databases to various cloud storage providers or local filesystems.

Designed to make database migrations easier, this project streamlines copying, backup, and restoration operations. It works both as a command-line tool for server automation and through a GUI for everyday development tasks like pulling production data into your local environment.

**Important Note:** This is a side project. It is not industrial-grade and might struggle with massive databases. Works great for development, testing, and smaller projects, but maybe don't bet your mission-critical production systems on it just yet...

## Features

- **Database Support**: PostgreSQL backup & restore
- **Storage Options**: S3 compatible storage or local filesystem
- **Compression**: Optional compression with configurable levels
- **Flexible Restoration**: Restore from specific backups or automatically use the latest one
- **Listing & Management**: List available backups with filtering options
- **Retention Management**: Define retention periods for automatic cleanup of old backups
- **SSH Tunneling**: Connect to remote databases through SSH tunnels
- **Environment Variable Support**: Configure via environment variables for seamless CI/CD integration
- **Cross-Platform**: Available for Linux and macOS

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

This CLI tool helps you back up, restore, and list PostgreSQL databases, with support for local or S3 storage and SSH tunneling.

## Commands

The CLI supports four main commands:

- `backup`: Create a database backup
- `restore`: Restore a database from backup
- `list`: List available backups
- `cleanup`: Delete backups older than a specified retention period

## Parameter Reference

### Global Parameters

| Parameter   | Description              | Default | Required | Environment Variable |
| ----------- | ------------------------ | ------- | -------- | -------------------- |
| `--help`    | Show help information    | -       | No       | -                    |
| `--version` | Show version information | -       | No       | -                    |

### Backup Command Parameters

```
cli backup [OPTIONS] --database <DATABASE> [--storage-type <TYPE>] [--retention <PERIOD>] [--other-options]
```

### Restore Command Parameters

```
cli restore [OPTIONS] --database <DATABASE> [--storage-type <TYPE>] [--filename <FILENAME> | --latest] [--other-options]
```

### List Command Parameters

```
cli list [OPTIONS] [--database <DATABASE>] [--storage-type <TYPE>] [--other-options]
```

### Cleanup Command Parameters

```
cli cleanup [OPTIONS] --retention <PERIOD> [--database <DATABASE>] [--storage-type <TYPE>] [--dry-run]
```

### Source Parameters (Database Connection)

| Parameter          | Description                    | Default     | Required                          | Environment Variable |
| ------------------ | ------------------------------ | ----------- | --------------------------------- | -------------------- |
| `--source-type`    | Database type                  | `postgres`  | No                                | -                    |
| `--source-name`    | Name identifier for the source | `default`   | No                                | -                    |
| `--database`, `-d` | Database name                  | -           | Yes                               | -                    |
| `--host`, `-H`     | Database host                  | `localhost` | No                                | -                    |
| `--port`, `-p`     | Database port                  | `5432`      | No                                | -                    |
| `--username`, `-u` | Database username              | `postgres`  | No                                | -                    |
| `--password`       | Database password              | -           | No                                | `PGPASSWORD`         |
| `--use-ssh-tunnel` | Enable SSH tunneling           | `false`     | No                                | -                    |
| `--ssh-key-path`   | Path to SSH private key        | -           | Only if `--use-ssh-tunnel` is set | -                    |
| `--ssh-username`   | SSH username                   | -           | Only if `--use-ssh-tunnel` is set | -                    |

### Storage Parameters

| Parameter        | Description                            | Default   | Required | Environment Variable |
| ---------------- | -------------------------------------- | --------- | -------- | -------------------- |
| `--storage-type` | Storage backend type (`s3` or `local`) | `s3`      | No       | -                    |
| `--storage-name` | Name identifier for storage            | `default` | No       | -                    |
| `--prefix`       | Optional prefix for backup files       | -         | No       | -                    |

#### S3 Storage Parameters

| Parameter      | Description     | Default     | Required   | Environment Variable                    |
| -------------- | --------------- | ----------- | ---------- | --------------------------------------- |
| `--bucket`     | S3 bucket name  | -           | Yes for S3 | `S3_BUCKET`                             |
| `--region`     | S3 region       | `us-east-1` | No         | `S3_REGION`                             |
| `--endpoint`   | S3 endpoint URL | -           | Yes for S3 | `S3_ENDPOINT`                           |
| `--access-key` | S3 access key   | -           | Yes for S3 | `S3_ACCESS_KEY_ID`, `S3_ACCESS_KEY`     |
| `--secret-key` | S3 secret key   | -           | Yes for S3 | `S3_SECRET_ACCESS_KEY`, `S3_SECRET_KEY` |

#### Local Storage Parameters

| Parameter | Description                     | Default | Required      | Environment Variable |
| --------- | ------------------------------- | ------- | ------------- | -------------------- |
| `--root`  | Root directory path for backups | -       | Yes for local | -                    |

### Backup-Specific Parameters

| Parameter             | Description                                              | Default | Required | Environment Variable |
| --------------------- | -------------------------------------------------------- | ------- | -------- | -------------------- |
| `--compression`, `-c` | Compression level (0-9)                                  | -       | No       | -                    |
| `--retention`, `-r`   | Retention period for the backup (e.g. '30d', '1w', '6m') | -       | No       | -                    |

### Restore-Specific Parameters

| Parameter               | Description                               | Default | Required                                   | Environment Variable |
| ----------------------- | ----------------------------------------- | ------- | ------------------------------------------ | -------------------- |
| `--filename`, `-f`      | Specific backup file to restore           | -       | One of `--filename` or `--latest` required | -                    |
| `--drop-database`, `-d` | Drop database if it exists before restore | -       | No                                         | -                    |
| `--latest`              | Use the most recent backup                | `false` | One of `--filename` or `--latest` required | -                    |

### List-Specific Parameters

| Parameter          | Description                                   | Default | Required | Environment Variable |
| ------------------ | --------------------------------------------- | ------- | -------- | -------------------- |
| `--database`, `-d` | Filter backups for specific database          | -       | No       | -                    |
| `--latest-only`    | Show only the latest backup for each database | `false` | No       | -                    |
| `--limit`, `-l`    | Maximum number of backups to list             | `10`    | No       | -                    |

### Cleanup-Specific Parameters

| Parameter           | Description                                          | Default | Required | Environment Variable |
| ------------------- | ---------------------------------------------------- | ------- | -------- | -------------------- |
| `--retention`, `-r` | Retention period (e.g. '30d', '1w', '6m', '1y')      | -       | Yes      | -                    |
| `--dry-run`         | Show what would be deleted without actually deleting | `false` | No       | -                    |
| `--database`, `-d`  | Filter cleanup for specific database                 | -       | No       | -                    |

## Retention Format

The retention period defines how long backups should be kept before being automatically deleted. The format is:

- `Nd`: N days (e.g., `30d` for 30 days)
- `Nw`: N weeks (e.g., `4w` for 4 weeks)
- `Nm`: N months (e.g., `6m` for 6 months, calculated as 30 days per month)
- `Ny`: N years (e.g., `1y` for 1 year, calculated as 365 days)

## Examples

### Create a backup to S3

```bash
cli backup \
  --database my_database \
  --host db.example.com \
  --username dbuser \
  --password mypassword \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAIOSFODNN7EXAMPLE \
  --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
```

### Create a backup with retention period

```bash
cli backup \
  --database my_database \
  --retention 30d \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAIOSFODNN7EXAMPLE \
  --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
```

### Create a backup to local storage

```bash
cli backup \
  --database my_database \
  --storage-type local \
  --root /path/to/backups
```

### Backup with SSH tunnel

```bash
cli backup \
  --database my_database \
  --host 10.0.0.5 \
  --use-ssh-tunnel \
  --ssh-username ssh_user \
  --ssh-key-path ~/.ssh/id_rsa \
  --storage-type local \
  --root /path/to/backups
```

### Restore the latest backup

```bash
cli restore \
  --database my_database \
  --latest \
  --storage-type local \
  --root /path/to/backups
```

### List available backups

```bash
cli list \
  --storage-type local \
  --root /path/to/backups \
  --limit 20
```

### Clean up old backups

```bash
cli cleanup \
  --retention 30d \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAIOSFODNN7EXAMPLE \
  --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
```

### Preview which backups would be deleted (dry run)

```bash
cli cleanup \
  --retention 14d \
  --dry-run \
  --storage-type local \
  --root /path/to/backups
```

### Clean up old backups for a specific database

```bash
cli cleanup \
  --retention 60d \
  --database production_db \
  --storage-type s3 \
  --bucket my-backups
```

## Backup Naming Convention

Backups are automatically named using the format:

```
{source-name}-{database}-{timestamp}-{uuid}.gz
```

For example:

```
default-mydb-2023-10-15-084531-a7bf34.gz
```

## Setting Up Cron Jobs

Add to `/etc/cron.d/database-backups`:

```
# Daily PostgreSQL backup at 2:00 AM with 30-day retention
0 2 * * * root S3_BUCKET=my-backup-bucket PGPASSWORD=secret /usr/local/bin/vprs3bkp backup --database mydb --username dbuser --host db.example.com --storage-type s3 --retention 30d

# Weekly cleanup of backups older than 30 days
0 3 * * 0 root S3_BUCKET=my-backup-bucket /usr/local/bin/vprs3bkp cleanup --retention 30d --storage-type s3
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
