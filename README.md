# vprs3bkp - Database Backup to S3

A robust, efficient CLI tool written in Rust to backup PostgreSQL and MySQL/MariaDB databases directly to S3-compatible storage.

## Features

- Backup PostgreSQL databases to S3 with automatic version detection
- Backup MySQL/MariaDB databases to S3
- Backup local folders with parallel uploads to S3
- File filtering by pattern, size, and type
- Support for custom S3-compatible storage providers
- Automatic PostgreSQL version detection and compatibility handling
- Docker fallback for PostgreSQL version mismatches
- Configurable compression levels
- Automatic backup file naming with timestamps and UUIDs
- Verbose logging option for debugging
- Environment variable support for secrets and configuration
- SSL verification toggle for development environments

## Installation

### Quick Installation

#### Linux

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install.sh | sudo bash
```

#### macOS

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install.sh | bash
```

Note: macOS users may need to remove `sudo` depending on their permission settings.

### Install with Dependencies

To also install required system dependencies (PostgreSQL client, MySQL client, gzip):

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install.sh | sudo bash -s -- --with-deps
```

### Static Build Installation (Better Compatibility)

For environments where dynamic linking might be an issue:

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install.sh | sudo bash -s -- --musl
```

### Install from Source (requires Rust toolchain)

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/vprs3bkp/main/install.sh | sudo bash -s -- --from-source
```

Or manually:

```bash
# Clone the repository
git clone https://github.com/vpr-group/vprs3bkp.git
cd vprs3bkp

# Build the project
cargo build --release

# Install the binary
sudo cp target/release/vprs3bkp /usr/local/bin/
```

## Usage

### PostgreSQL Backup

Basic usage:

```bash
vprs3bkp --bucket my-backup-bucket postgres --database mydb --username dbuser
```

With all options:

```bash
vprs3bkp \
  --bucket my-backup-bucket \
  --region us-west-2 \
  --prefix db-backups/production \
  --endpoint https://custom-s3-provider.com \
  --access-key YOUR_ACCESS_KEY \
  --secret-key YOUR_SECRET_KEY \
  --verbose \
  postgres \
  --database mydb \
  --host db.example.com \
  --port 5432 \
  --username dbuser \
  --password dbpassword \
  --compression 9 \
  --force-docker
```

### MySQL/MariaDB Backup

Basic usage:

```bash
vprs3bkp --bucket my-backup-bucket mysql --database mydb --username dbuser
```

With all options:

```bash
vprs3bkp \
  --bucket my-backup-bucket \
  --region us-west-2 \
  --prefix db-backups/production \
  --endpoint https://custom-s3-provider.com \
  --access-key YOUR_ACCESS_KEY \
  --secret-key YOUR_SECRET_KEY \
  --verbose \
  mysql \
  --database mydb \
  --host db.example.com \
  --port 3306 \
  --username dbuser \
  --password dbpassword \
  --compression 9
```

### Folder Backup

Basic usage:

```bash
vprs3bkp --bucket my-backup-bucket folder --path /path/to/backup
```

With all options:

```bash
vprs3bkp \
  --bucket my-backup-bucket \
  --region us-west-2 \
  --prefix folder-backups \
  --verbose \
  folder \
  --path /path/to/important/files \
  --compress \
  --compression-level 9 \
  --concurrency 8 \
  --include "*.{jpg,png,pdf}" \
  --exclude "temp/*" \
  --skip-larger-than 100 \
  --add-timestamp
```

#### Folder Backup Options

- `--path`: Local folder path to backup (required)
- `--compress`: Enable gzip compression for each file
- `--compression-level`: Set compression level (0-9, default: 6)
- `--concurrency`: Number of parallel uploads (1-100, default: 4)
- `--include`: Only include files matching these patterns (glob syntax, can be specified multiple times)
- `--exclude`: Exclude files matching these patterns (glob syntax, can be specified multiple times)
- `--skip-larger-than`: Skip files larger than this size in MB
- `--add-timestamp`: Add timestamp to the S3 prefix for better organization

## Environment Variables

All command-line options can be specified through environment variables:

### S3 Configuration

```bash
export S3_REGION=us-east-1
export S3_BUCKET=my-backup-bucket
export S3_PREFIX=backups/databases
export S3_ENDPOINT=https://custom-s3-provider.com
export S3_ACCESS_KEY_ID=your_access_key
export S3_SECRET_ACCESS_KEY=your_secret_key
```

### Database Credentials

```bash
# PostgreSQL
export PGPASSWORD=your_postgres_password

# MySQL
export MYSQL_PWD=your_mysql_password
```

## Docker Support for PostgreSQL

The tool includes an intelligent version detection system that:

1. Detects your PostgreSQL server version
2. Checks compatibility with local `pg_dump`
3. Automatically uses Docker with matching PostgreSQL version when needed
4. Falls back to local `pg_dump` if Docker is unavailable

You can force Docker usage with the `--force-docker` flag.

## AWS Authentication

The tool uses the AWS SDK's credential provider chain, which checks for credentials in this order:

1. Command-line arguments (`--access-key`, `--secret-key`)
2. Environment variables (`S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`)
3. AWS shared credentials file (`~/.aws/credentials`)
4. IAM role for Amazon EC2 or ECS task role

## S3-Compatible Storage Providers

To use with S3-compatible storage providers (MinIO, DigitalOcean Spaces, etc.):

```bash
vprs3bkp \
  --bucket my-bucket \
  --endpoint https://minio.example.com \
  --access-key ACCESS_KEY \
  --secret-key SECRET_KEY \
  postgres \
  --database mydb \
  --username dbuser
```

## Setting Up Cron Jobs

Add to `/etc/cron.d/database-backups`:

```
# Daily PostgreSQL backup at 2:00 AM
0 2 * * * root S3_BUCKET=my-backup-bucket PGPASSWORD=secret /usr/local/bin/vprs3bkp postgres --database mydb --username dbuser --host db.example.com

# Daily MySQL backup at 3:00 AM
0 3 * * * root S3_BUCKET=my-backup-bucket MYSQL_PWD=secret /usr/local/bin/vprs3bkp mysql --database mydb --username dbuser --host db.example.com

# Weekly folder backup on Sunday at 1:00 AM
0 1 * * 0 root S3_BUCKET=my-backup-bucket /usr/local/bin/vprs3bkp folder --path /var/www/html --compress --concurrency 8
```

## Backup File Structure

Backups are stored with the following path format:

```
s3://{bucket}/{prefix}/{db_type}/{db_name}-{date}-{time}-{uuid}.gz
```

Example:

```
s3://my-backup-bucket/vprs3bkp/postgres/mydb-2023-04-15-120135-a1b2c3d4.gz
```

## Prerequisites

- PostgreSQL client tools (`pg_dump` and `psql`) for PostgreSQL backups
- MySQL client tools (`mysqldump`) for MySQL backups
- Docker (optional, for version-compatible PostgreSQL backups)
- AWS credentials with `s3:PutObject` permissions
- Gzip for compression

### Installing Prerequisites on macOS

```bash
# Using Homebrew
brew install postgresql mysql-client

# Verify installation
psql --version
pg_dump --version
mysql --version
```

### Installing Prerequisites on Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y postgresql-client mysql-client gzip

# Verify installation
psql --version
pg_dump --version
mysql --version
```

## Troubleshooting

### Common Issues

1. **Connection refused**: Check database host, port, and firewall settings
2. **Access denied**: Verify database credentials
3. **S3 upload failed**: Check S3 credentials and permissions
4. **"Failed to execute psql command: no such file"** or **"pg_dump not found"**: These errors indicate that PostgreSQL client tools are not installed or not in your PATH. Install them with:
   - macOS: `brew install postgresql`
   - Ubuntu/Debian: `sudo apt-get install postgresql-client`
   - Or use Docker by adding the `--force-docker` flag to bypass local PostgreSQL client tools
5. **"version mismatch detected but Docker is not available"**: Install Docker or ensure your local PostgreSQL client version is compatible with your server

Use the `--verbose` flag for detailed logging:

```bash
vprs3bkp --verbose --bucket my-bucket postgres --database mydb --username dbuser
```

For development environments with self-signed certificates:

```bash
vprs3bkp --no-verify-ssl --endpoint https://dev-s3.example.com --bucket test-bucket postgres --database mydb --username dbuser
```

## License

MIT
