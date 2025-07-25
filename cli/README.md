# DBKP - Database Backup & Restore CLI

A professional command-line tool for backing up and restoring PostgreSQL and MySQL databases with support for multiple storage backends, workspaces, and interactive configuration.

## 🚀 Quick Start

### Installation

```bash
# Clone and build from source
git clone https://github.com/yourusername/vprs3bkp.git
cd vprs3bkp
cargo build --release

# Copy binary to PATH
sudo cp target/release/dbkp /usr/local/bin/
```

### First Run

The easiest way to get started is with interactive mode:

```bash
dbkp
```

This launches an interactive wizard that guides you through creating your first workspace and running backups.

## 📖 Usage Modes

### 1. Interactive Mode (Recommended for Beginners)

Simply run `dbkp` without arguments to enter the interactive wizard:

```bash
dbkp
```

**Features:**
- 🎯 Guided workspace creation
- 🔧 Step-by-step database configuration  
- 🌐 Storage setup with validation
- ✅ Real-time connection testing
- 📊 Backup/restore with progress indicators
- 🗂️ Workspace management

### 2. Workspace Mode (Recommended for Regular Use)

Save your configurations as workspaces for easy reuse:

```bash
# Use active workspace
dbkp backup --workspace myproject

# Use specific workspace  
dbkp restore --workspace production --latest
```

### 3. Direct Parameters (For Automation)

Specify all parameters directly in the command:

```bash
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location myapp-backups
```

## 🏗️ Commands Overview

| Command | Description |
|---------|-------------|
| `dbkp` | Launch interactive mode |
| `dbkp backup` | Create database backup |
| `dbkp restore` | Restore database from backup |
| `dbkp list` | List available backups |
| `dbkp cleanup` | Remove old backups |
| `dbkp workspace` | Manage workspaces |

## 🗂️ Workspace Management

### List Workspaces
```bash
dbkp workspace list
```

### Create/Switch Workspace
```bash
# Switch to existing workspace
dbkp workspace use production

# Delete workspace
dbkp workspace delete oldproject

# Show active workspace
dbkp workspace active
```

## 💾 Backup Operations

### Using Workspaces
```bash
# Backup with active workspace
dbkp backup --workspace myproject

# Backup with specific workspace
dbkp backup --workspace production
```

### Direct Parameters

**PostgreSQL to Local Storage:**
```bash
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --password secret \
  --storage-type local \
  --location /backups/myapp
```

**MySQL to S3:**
```bash
dbkp backup \
  --database-type mysql \
  --database myapp \
  --host db.example.com \
  --port 3306 \
  --username dbuser \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAIOSFODNN7EXAMPLE \
  --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
  --location myapp-backups
```

**With SSH Tunnel:**
```bash
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host 10.0.0.100 \
  --port 5432 \
  --username dbuser \
  --ssh-host bastion.example.com \
  --ssh-username ubuntu \
  --ssh-key-path ~/.ssh/id_rsa \
  --storage-type local \
  --location /backups
```

## 🔄 Restore Operations

### Interactive Restore

When using workspaces, you'll see a formatted list:

```
Available backups (newest first):
   1. 2024-01-15 14:30:22 UTC | 2.45MB | myapp-2024-01-15-143022-a1b2c3d4.gz
   2. 2024-01-15 12:15:18 UTC | 1.98MB | myapp-2024-01-15-121518-e5f6g7h8.gz
   3. 2024-01-14 09:45:12 UTC | 2.12MB | myapp-2024-01-14-094512-i9j0k1l2.gz
```

### Command Line Restore

```bash
# Restore latest backup
dbkp restore --workspace myproject --latest

# Restore specific backup
dbkp restore \
  --workspace myproject \
  --name myapp-2024-01-15-143022-a1b2c3d4.gz \
  --drop-database

# Direct parameters
dbkp restore \
  --database-type postgresql \
  --database myapp_restore \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type local \
  --location /backups \
  --latest
```

## 📋 List Backups

```bash
# Using workspace
dbkp list --workspace myproject

# Direct parameters - local storage
dbkp list \
  --storage-type local \
  --location /backups

# Direct parameters - S3 storage
dbkp list \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location myapp-backups
```

## 🧹 Cleanup Operations

```bash
# Using workspace
dbkp cleanup --workspace myproject --retention 30d

# Direct with dry run
dbkp cleanup \
  --storage-type s3 \
  --bucket my-backups \
  --endpoint https://s3.amazonaws.com \
  --access-key AKIAKEY \
  --secret-key SECRET \
  --location myapp-backups \
  --retention 7d \
  --dry-run
```

## 🔧 Parameter Reference

### Database Connection

| Parameter | Description | Required | Default |
|-----------|-------------|----------|---------|
| `--database-type` | Database type (`postgresql`, `mysql`) | Yes | - |
| `--database` | Database name | Yes | - |
| `--host` | Database host | Yes | - |
| `--port` | Database port | Yes | - |
| `--username` | Database username | Yes | - |
| `--password` | Database password | No | - |

### SSH Tunnel

| Parameter | Description | Required | Default |
|-----------|-------------|----------|---------|
| `--ssh-host` | SSH host | Yes (if using SSH) | - |
| `--ssh-username` | SSH username | Yes (if using SSH) | - |
| `--ssh-key-path` | SSH private key path | Yes (if using SSH) | - |

### Storage - Local

| Parameter | Description | Required | Default |
|-----------|-------------|----------|---------|
| `--storage-type` | Set to `local` | Yes | `local` |
| `--location` | Directory path | Yes | - |

### Storage - S3

| Parameter | Description | Required | Default |
|-----------|-------------|----------|---------|
| `--storage-type` | Set to `s3` | Yes | - |
| `--bucket` | S3 bucket name | Yes | - |
| `--endpoint` | S3 endpoint URL | Yes | - |
| `--access-key` | S3 access key | Yes | - |
| `--secret-key` | S3 secret key | Yes | - |
| `--location` | Prefix/folder in bucket | Yes | - |
| `--region` | S3 region | No | `us-east-1` |

### Restore Options

| Parameter | Description | Required | Default |
|-----------|-------------|----------|---------|
| `--name` | Specific backup to restore | No* | - |
| `--latest` | Use most recent backup | No* | `false` |
| `--drop-database` | Drop database before restore | No | `false` |

*Either `--name` or `--latest` is required for restore operations.

### Cleanup Options

| Parameter | Description | Required | Default |
|-----------|-------------|----------|---------|
| `--retention` | Keep backups newer than this | Yes | - |
| `--dry-run` | Show what would be deleted | No | `false` |

## 🌍 Environment Variables

| Variable | Description | CLI Equivalent |
|----------|-------------|----------------|
| `PGPASSWORD` | PostgreSQL password | `--password` |
| `S3_BUCKET` | S3 bucket name | `--bucket` |
| `S3_ENDPOINT` | S3 endpoint URL | `--endpoint` |
| `S3_ACCESS_KEY` | S3 access key | `--access-key` |
| `S3_SECRET_KEY` | S3 secret key | `--secret-key` |
| `S3_REGION` | S3 region | `--region` |

### Using Environment Variables

```bash
# Set environment variables
export PGPASSWORD=mysecret
export S3_BUCKET=my-backups
export S3_ACCESS_KEY=AKIAKEY
export S3_SECRET_KEY=SECRET

# Use in commands (parameters not needed)
dbkp backup \
  --database-type postgresql \
  --database myapp \
  --host localhost \
  --port 5432 \
  --username dbuser \
  --storage-type s3 \
  --endpoint https://s3.amazonaws.com \
  --location myapp-backups
```

## 🎨 User Interface Features

### Animated Spinners

All operations show animated progress indicators with color changes:

```
| Connecting to database...     (Red)
/ Database connected...         (Yellow)  
- Testing storage connection... (Green)
\ Starting backup...            (Cyan)
[SUCCESS] Backup completed successfully!
```

### Colored Output

- **Success messages**: Green `[SUCCESS]`
- **Error messages**: Red `[ERROR]`
- **Info messages**: Cyan `[INFO]`
- **Active workspace**: Green + Bold
- **Backup listings**: Formatted with dates and sizes

### Professional Interface

- Clean ASCII-only characters
- Consistent formatting throughout
- Clear progress feedback
- Detailed error messages
- No emoji clutter

## 📝 Backup Naming Convention

Backups are automatically named with timestamps:

```
{database-name}-{YYYY-MM-DD-HHMMSS}-{uuid}.{extension}
```

Example:
```
myapp-2024-01-15-143022-a1b2c3d4.gz
```

## 🔄 Retention Periods

Specify how long to keep backups:

- `30d` - 30 days
- `4w` - 4 weeks  
- `6m` - 6 months (calculated as 30 days each)
- `1y` - 1 year (calculated as 365 days)

Examples:
```bash
# Keep backups for 30 days
dbkp cleanup --workspace myproject --retention 30d

# Keep backups for 6 months
dbkp cleanup --workspace myproject --retention 6m
```

## 🤖 Automation Examples

### Cron Job

```bash
# Daily backup at 2 AM
0 2 * * * /usr/local/bin/dbkp backup --workspace production 2>&1 | logger -t dbkp

# Weekly cleanup 
0 3 * * 0 /usr/local/bin/dbkp cleanup --workspace production --retention 30d 2>&1 | logger -t dbkp
```

### Systemd Timer

Create `/etc/systemd/system/dbkp-backup.service`:

```ini
[Unit]
Description=Database Backup
After=network.target

[Service]
Type=oneshot
User=dbkp
ExecStart=/usr/local/bin/dbkp backup --workspace production
```

Create `/etc/systemd/system/dbkp-backup.timer`:

```ini
[Unit]
Description=Run database backup daily
Requires=dbkp-backup.service

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

Enable:
```bash
sudo systemctl enable dbkp-backup.timer
sudo systemctl start dbkp-backup.timer
```

### CI/CD Pipeline

**GitHub Actions:**
```yaml
name: Database Backup
on:
  schedule:
    - cron: '0 2 * * *'

jobs:
  backup:
    runs-on: ubuntu-latest
    steps:
      - name: Backup Database
        run: |
          dbkp backup \
            --database-type postgresql \
            --database ${{ secrets.DB_NAME }} \
            --host ${{ secrets.DB_HOST }} \
            --port 5432 \
            --username ${{ secrets.DB_USER }} \
            --password ${{ secrets.DB_PASSWORD }} \
            --storage-type s3 \
            --bucket ${{ secrets.S3_BUCKET }} \
            --endpoint ${{ secrets.S3_ENDPOINT }} \
            --access-key ${{ secrets.S3_ACCESS_KEY }} \
            --secret-key ${{ secrets.S3_SECRET_KEY }} \
            --location production-backups
```

**GitLab CI:**
```yaml
backup:
  stage: backup
  script:
    - dbkp backup --workspace production
  only:
    - schedules
  variables:
    PGPASSWORD: $DB_PASSWORD
    S3_ACCESS_KEY: $S3_ACCESS_KEY
    S3_SECRET_KEY: $S3_SECRET_KEY
```

### Docker Usage

**Dockerfile:**
```dockerfile
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    postgresql-client \
    mysql-client \
    curl

# Install dbkp
COPY dbkp /usr/local/bin/
RUN chmod +x /usr/local/bin/dbkp

# Create backup script
COPY backup.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/backup.sh

CMD ["/usr/local/bin/backup.sh"]
```

**backup.sh:**
```bash
#!/bin/bash
set -e

dbkp backup \
    --database-type postgresql \
    --database ${DB_NAME} \
    --host ${DB_HOST} \
    --port ${DB_PORT:-5432} \
    --username ${DB_USER} \
    --password ${DB_PASSWORD} \
    --storage-type s3 \
    --bucket ${S3_BUCKET} \
    --endpoint ${S3_ENDPOINT} \
    --access-key ${S3_ACCESS_KEY} \
    --secret-key ${S3_SECRET_KEY} \
    --location ${S3_LOCATION}
```

**docker-compose.yml:**
```yaml
version: '3.8'
services:
  db-backup:
    build: .
    environment:
      - DB_NAME=myapp
      - DB_HOST=database
      - DB_USER=dbuser
      - DB_PASSWORD=secret
      - S3_BUCKET=my-backups
      - S3_ENDPOINT=https://s3.amazonaws.com
      - S3_ACCESS_KEY=AKIAKEY
      - S3_SECRET_KEY=SECRET
      - S3_LOCATION=production-backups
    depends_on:
      - database
```

## 🐛 Troubleshooting

### Common Issues

**Database Connection Failed**
```bash
[ERROR] Failed to connect to database
```
**Solutions:**
- Verify host, port, username, password
- Check database server is running
- Test connection: `psql -h host -p port -U username -d database`
- Check firewall settings

**Storage Connection Failed**
```bash
[ERROR] Failed to connect to storage
```
**Solutions:**
- Verify S3 credentials and permissions
- Check bucket exists and is accessible
- Test endpoint URL
- For local storage, verify directory permissions

**No Backups Found**
```bash
[ERROR] No backups found in storage
```
**Solutions:**
- Check storage location/bucket path
- Verify backup naming convention
- List files manually to confirm presence

**Permission Denied**
```bash
[ERROR] Permission denied
```
**Solutions:**
- Check file/directory permissions
- Verify user has read/write access
- For S3, check IAM permissions

**pg_dump/mysqldump Not Found**
```bash
[ERROR] Failed to execute pg_dump: command not found
```
**Solutions:**
- Install PostgreSQL client: `sudo apt-get install postgresql-client`
- Install MySQL client: `sudo apt-get install mysql-client`
- On macOS: `brew install postgresql mysql-client`

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug dbkp backup --workspace myproject
```

### Test Connections

Use interactive mode to test connections before running automated backups:

```bash
dbkp
# Create workspace with your settings
# Test backup/restore manually
```

### Workspace File Locations

Workspaces are stored in:
- Linux: `~/.config/dbkp/workspaces.json`
- macOS: `~/Library/Application Support/dbkp/workspaces.json`

### Manual Workspace Editing

You can manually edit the workspace file if needed:

```json
{
  "workspaces": [
    {
      "name": "production",
      "database": {
        "connection_type": "PostgreSql",
        "database": "myapp",
        "host": "db.example.com",
        "port": 5432,
        "username": "dbuser",
        "password": null,
        "ssh_tunnel": null
      },
      "storage": {
        "S3": {
          "name": "production-s3",
          "bucket": "my-backups",
          "region": "us-east-1",
          "endpoint": "https://s3.amazonaws.com",
          "access_key": "AKIAKEY",
          "secret_key": "SECRET",
          "location": "production-backups"
        }
      },
      "created_at": "2024-01-15T10:30:00Z",
      "last_used": null
    }
  ],
  "active_workspace": "production"
}
```

## 📊 Performance Tips

### Large Databases

For databases > 100GB:
- Use direct network connection (avoid SSH tunnels)
- Run backups during low-traffic periods
- Consider using parallel backup tools for very large databases
- Monitor disk space on both source and destination

### Network Optimization

- Use compression for remote backups
- Consider regional S3 endpoints for faster uploads
- Test network bandwidth before scheduling frequent backups

### Storage Optimization

- Use lifecycle policies on S3 for automatic archival
- Implement retention policies to manage storage costs
- Monitor backup sizes over time

## 🔗 Related Tools

- **PostgreSQL**: [pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html), [Barman](https://pgbarman.org)
- **MySQL**: [mysqldump](https://dev.mysql.com/doc/refman/8.0/en/mysqldump.html)
- **Enterprise**: [pgbackrest](https://pgbackrest.org)
- **Monitoring**: Integrate with your monitoring stack for backup success/failure alerts

## 📄 License

MIT License - See LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📞 Support

- **GitHub Issues**: Report bugs or request features
- **Discussions**: Community support and questions

---

**Happy backing up! 🚀**