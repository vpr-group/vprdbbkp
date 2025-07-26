# DB Backup - Database Backup & Restore Utility

A simple tool for backing up databases to various cloud storage providers or local filesystems.

Designed to make database migrations easier, this project streamlines copying, backup, and restoration operations. It works both as a command-line tool for server automation and through a GUI for everyday development tasks like pulling production data into your local environment.

**Important Note:** This is a side project. It is not an industrial-grade solution. It only provides logical backup for the moment and might struggle with massive databases. Works great for development, testing, and smaller projects, but maybe don't bet your mission-critical production systems on it just yet...

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
- **Connection Options**: Direct connections or SSH tunneling
- **Version Detection**: Automatic PostgreSQL version detection and compatibility
- **Connection Pooling**: Efficient connection management with configurable pooling

### Storage Backends
- **S3-Compatible Storage**: Amazon S3, MinIO, DigitalOcean Spaces, and other S3-compatible providers
- **Local Filesystem**: Store backups on local or network-mounted filesystems
- **Flexible Configuration**: Multiple storage configurations per project

### Backup & Restore Operations
- **Streaming Architecture**: Memory-efficient streaming for large databases without loading everything into memory
- **Logical Backups**: Full schema and data backup using `pg_dump`
- **Flexible Restore Options**: Restore to same or different database with optional database recreation
- **Connection Termination**: Automatic handling of active connections during restore
- **Data Integrity**: Built-in validation and error handling

### Management & Organization
- **Workspace System**: Save and reuse database and storage configurations
- **Backup Listing**: View available backups with timestamps and sizes
- **Retention Policies**: Automatic cleanup of old backups (30d, 4w, 6m, 1y formats)
- **Dry Run Mode**: Preview cleanup operations before execution
- **Backup Naming**: Consistent naming with timestamps and UUIDs

### Connectivity & Security
- **SSH Tunneling**: Secure connections to remote databases through SSH
- **Private Key Authentication**: Support for SSH key-based authentication
- **Environment Variable Support**: Secure credential management via environment variables
- **Connection Testing**: Built-in connection validation before operations

### User Experience
- **Interactive Mode**: Guided setup wizard for easy configuration
- **Progress Indicators**: Real-time feedback with animated spinners
- **Colored Output**: Professional colored terminal output
- **Error Handling**: Detailed error messages with context
- **Cross-Platform**: Support for Linux and macOS

### Automation & Integration
- **CLI Automation**: Full command-line interface for scripts and CI/CD
- **Environment Variables**: Complete environment variable support for automated deployments
- **Cron Job Ready**: Designed for scheduled backup operations
- **Docker Compatible**: Works in containerized environments
- **CI/CD Integration**: Examples for GitHub Actions, GitLab CI, and other platforms

### Performance & Efficiency
- **Streaming Processing**: Handles large databases efficiently with constant memory usage
- **Configurable Buffers**: Tunable buffer sizes for optimal performance
- **Connection Pooling**: Efficient database connection management
- **Binary Management**: Automatic download and installation of required PostgreSQL utilities
- **Version Compatibility**: Support for multiple PostgreSQL versions

### Monitoring & Debugging
- **Comprehensive Logging**: Detailed logging for troubleshooting
- **Connection Validation**: Pre-flight checks before operations
- **Storage Testing**: Validate storage connectivity before backup/restore
- **Debug Mode**: Enhanced logging for development and troubleshooting

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
- **PostgreSQL Module**: Specialized PostgreSQL implementation with streaming support
- **Storage Modules**: S3 and local filesystem storage implementations
- **SSH Tunneling**: Secure remote database connections

## Use Cases

### Development & Testing
- Pull production data to local development environments
- Create test data snapshots for consistent testing
- Quick database migrations between environments

### Small to Medium Production
- Automated daily/weekly backups with retention policies
- Database migrations and deployments
- Disaster recovery for smaller applications

### CI/CD Integration
- Automated backup verification in deployment pipelines
- Database state management in testing environments
- Environment synchronization across deployment stages

## Limitations

- **Logical Backups Only**: Does not support physical/binary backups
- **Single Database Focus**: Optimized for individual database operations
- **Not Industrial-Grade**: Suitable for development and smaller production use cases
- **PostgreSQL Focused**: Currently optimized primarily for PostgreSQL

For enterprise-grade solutions with physical backups, point-in-time recovery, and high-availability features, consider [Barman](https://pgbarman.org) or [pgbackrest](https://pgbackrest.org).

## License

MIT License - see LICENSE file for details.
