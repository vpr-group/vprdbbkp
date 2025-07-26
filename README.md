# DB Backup - Database Backup & Restore Utility

A simple tool for backing up databases to various cloud storage providers or local filesystems.

Designed to make database migrations easier, this project streamlines copying, backup, and restoration operations. It works both as a command-line tool for server automation and through a GUI for everyday development tasks like pulling production data into your local environment.

**Important Note:** This is a side project used currently as an internal tool. It is not an industrial-grade solution. It only provides logical backup for the moment and might struggle with massive databases. Works great for development, testing, and smaller projects, but maybe don't bet your mission-critical production systems on it just yet...

If you need more advanced tools please check [Barman](https://pgbarman.org) or [pgbackrest](https://pgbackrest.org).

## Installation

### Quick Installation

#### Linux

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/dbkp/main/install-cli.sh | sudo bash
```

#### macOS

```bash
curl -sSL https://raw.githubusercontent.com/vpr-group/dbkp/main/install-cli.sh | bash
```

Note: macOS users may need to remove `sudo` depending on their permission settings.

### Install from Source (requires Rust toolchain)

```bash
# Clone the repository
git clone https://github.com/vpr-group/dbkp.git
cd dbkp

# Build the project
cd cli
cargo build --release

# Install the binary
sudo cp target/release/cli /usr/local/bin/dbkp
```

## GUI Application

This project also provides a GUI application for easier visual management of backups and databases. The GUI offers the same functionality as the CLI with a user-friendly interface.

**Note**: The GUI application currently requires manual building and is not distributed as pre-built binaries.

### Building the GUI

To build and run the GUI application:

```bash
# Clone the repository (if not already done)
git clone https://github.com/vpr-group/dbkp.git
cd dbkp

# Build the GUI application
cd app
# Follow build instructions in /app directory
```

For detailed GUI setup instructions, features, and usage, see the [GUI Documentation](/app).

## CLI Documentation

For detailed CLI usage, commands, parameters, and examples, see the [CLI Documentation](/cli/README.md).

The CLI supports multiple usage modes:
- **Interactive Mode**: Guided wizard for beginners
- **Workspace Mode**: Saved configurations for regular use
- **Direct Parameters**: Full command-line control for automation

## Features

### Database Support
- **PostgreSQL**: Full backup and restore support with streaming architecture
- **Version Detection**: Automatic PostgreSQL version detection and compatibility

### Storage Backends
- **S3-Compatible Storage**: Amazon S3, MinIO, DigitalOcean Spaces, and other S3-compatible providers
- **Local Filesystem**: Store backups on local or network-mounted filesystems

### Backup & Restore Operations
- **Streaming Architecture**: Memory-efficient streaming for large databases without loading everything into memory
- **Logical Backups**: Full schema and data backup using `pg_dump`

### User Experience
- **Interactive Mode**: Guided setup wizard for easy configuration

### Automation & Integration
- **CLI Automation**: Full command-line interface for scripts and CI/CD
- **Cron Job Ready**: Designed for scheduled backup operations
- **Docker Compatible**: Works in containerized environments

## Quick Start Example

```bash
# Interactive mode (recommended for first-time users)
dbkp

# Direct backup to S3
dbkp backup \
  --database myapp \
  --host localhost \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com

# Restore latest backup
dbkp restore \
  --database myapp \
  --storage-type s3 \
  --bucket my-backups \
  --latest
```

## Architecture

The project is organized into several modules:

- **Core Library** (`/core`): Database connections, backup/restore logic, and storage backends
- **CLI Tool** (`/cli`): Command-line interface with interactive and direct modes

## Use Cases

### Development & Testing
- Pull production data to local development environments
- Create test data snapshots for consistent testing
- Quick database migrations between environments

### Small to Medium Production
- Automated daily/weekly backups with retention policies
- Database migrations and deployments
- Disaster recovery for smaller applications

## Limitations

- **Logical Backups Only**: Does not support physical/binary backups
- **Single Database Focus**: Optimized for individual database operations
- **Not Industrial-Grade**: Suitable for development and smaller to medium production use cases
- **PostgreSQL Focused**: Currently optimized primarily for PostgreSQL

For enterprise-grade solutions with physical backups, point-in-time recovery, and high-availability features, consider [Barman](https://pgbarman.org) or [pgbackrest](https://pgbackrest.org).

## License

MIT License - see LICENSE file for details.
