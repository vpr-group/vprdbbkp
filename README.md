# Database Backup CLI Tool

A simple, efficient CLI tool written in Rust to backup PostgreSQL and MySQL databases directly to S3 buckets.

## Features

- Backup PostgreSQL databases to S3
- Backup MySQL/MariaDB databases to S3
- Backup folders with parallel uploads to S3
- File filtering by pattern, size, and type
- Optional compression for all backup types
- Configurable concurrency for large folder backups
- Simple installation on Linux servers
- Configurable S3 bucket and prefix
- AWS credentials from environment/config
- Automatic naming of backup files with timestamps

## Quick Installation

### Install pre-built binary

```bash
curl -sSL https://raw.githubusercontent.com/yourusername/vprs3bkp/main/install.sh | sudo bash
```

### Install with dependencies

```bash
curl -sSL https://raw.githubusercontent.com/yourusername/vprs3bkp/main/install.sh | sudo bash --with-deps
```

### Install from source (requires Rust)

```bash
curl -sSL https://raw.githubusercontent.com/yourusername/vprs3bkp/main/install.sh | sudo bash --from-source
```

## Usage

### Basic PostgreSQL Backup

```bash
vprs3bkp --bucket my-backup-bucket postgres --database mydb --username dbuser
```

### Basic MySQL Backup

```bash
vprs3bkp --bucket my-backup-bucket mysql --database mydb --username dbuser
```

### Full Options

```bash
# PostgreSQL with all options
vprs3bkp \
  --bucket my-backup-bucket \
  --region us-west-2 \
  --prefix db-backups/production \
  --verbose \
  postgres \
  --database mydb \
  --host db.example.com \
  --port 5432 \
  --username dbuser \
  --compression 9

# MySQL with all options
vprs3bkp \
  --bucket my-backup-bucket \
  --region us-west-2 \
  --prefix db-backups/production \
  --verbose \
  mysql \
  --database mydb \
  --host db.example.com \
  --port 3306 \
  --username dbuser \
  --compression 9

# Folder backup with all options
vprs3bkp \
  --bucket my-backup-bucket \
  --region us-west-2 \
  --prefix folder-backups \
  --verbose \
  folder \
  --path /path/to/important/files \
  --concurrency 20 \
  --compress \
  --compression-level 6 \
  --skip-larger-than 100 \
  --include "*.{jpg,png,pdf}" \
  --exclude "temp/*"
```

### Using Environment Variables

```bash
# AWS credentials and region
export S3_ACCESS_KEY_ID=your_access_key
export S3_SECRET_ACCESS_KEY=your_secret_key
export S3_REGION=us-east-1

# Database passwords
export PGPASSWORD=your_postgres_password
export MYSQL_PWD=your_mysql_password

# Bucket configuration
export S3_BUCKET=my-backup-bucket
export S3_PREFIX=backups/daily

# Run backup with minimal CLI arguments
vprs3bkp postgres --database mydb --username dbuser
```

### Setting Up Cron Jobs

Add to `/etc/cron.d/database-backups`:

````
# Daily PostgreSQL backup at 2:00 AM
0 2 * * * root S3_REGION=us-east-1 S3_BUCKET=my-backup-bucket PGPASSWORD=secret /usr/local/bin/vprs3bkp postgres --database mydb --username dbuser --host db.example.com

# Daily MySQL backup at 3:00 AM
0 3 * * * root S3_REGION=us-east-1 S3_BUCKET=my-backup-bucket MYSQL_PWD=secret /usr/local/bin/vprs3bkp mysql --database mydb --username dbuser --host db.example.com

# Weekly folder backup on Sunday at 1:00 AM
0 1 * * 0 root S3_REGION=us-east-1 S3_BUCKET=my-backup-bucket /usr/local/bin/vprs3bkp folder --path /var/www/html --compress --concurrency 5
``` --username dbuser --host db.example.com
````

## Prerequisites

- PostgreSQL client tools (pg_dump) for PostgreSQL backups
- MySQL client tools (mysqldump) for MySQL backups
- AWS credentials configured (environment variables or `~/.aws/credentials`)
- Appropriate S3 permissions (s3:PutObject)

## Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/vprs3bkp.git
cd vprs3bkp

# Build the project
cargo build --release

# Run the binary
./target/release/vprs3bkp --help
```

## Configuration

The tool uses the AWS SDK's default credential provider chain, which looks for credentials in the following order:

1. Environment variables: `S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY`
2. AWS credentials file: `~/.aws/credentials`
3. IAM role for Amazon EC2 or ECS task role

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
# vprs3bkp
# vprs3bkp
# vprs3bkp
# vprs3bkp
# vprs3bkp
