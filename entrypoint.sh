#!/bin/bash
# entrypoint.sh - Docker entrypoint for vprs3bkp

set -e

# Function to show usage
show_usage() {
    echo "vprs3bkp Docker Container"
    echo ""
    echo "Usage: docker run [OPTIONS] vprs3bkp:latest [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  backup    Perform database backup (default)"
    echo "  restore   Restore database from backup"
    echo "  list      List available backups"
    echo "  version   Show vprs3bkp version"
    echo "  help      Show this help"
    echo ""
    echo "Environment Variables:"
    echo "  DATABASE_TYPE    Database type (postgresql, mysql) [default: postgresql]"
    echo "  DATABASE         Database name (required)"
    echo "  HOST             Database host [default: localhost]"
    echo "  PORT             Database port [default: 5432]"
    echo "  USERNAME         Database username (required)"
    echo "  PASSWORD         Database password (required)"
    echo "  STORAGE_TYPE     Storage type (s3, local) [default: s3]"
    echo "  LOCATION         Backup location/path"
    echo "  BUCKET           S3 bucket name (required for S3)"
    echo "  REGION           S3 region [default: us-east-1]"
    echo "  ENDPOINT         S3 endpoint URL (for custom S3 providers)"
    echo "  ACCESS_KEY       S3 access key (required for S3)"
    echo "  SECRET_KEY       S3 secret key (required for S3)"
    echo "  BACKUP_NAME      Custom backup name [default: auto-generated]"
    echo "  FORMAT           Backup format [default: custom]"
    echo "  COMPRESS         Enable compression [default: true]"
    echo "  VERBOSE          Enable verbose output [default: false]"
    echo ""
    echo "Example:"
    echo "  docker run -e DATABASE=mydb -e USERNAME=user -e PASSWORD=pass \\"
    echo "             -e BUCKET=my-bucket -e ACCESS_KEY=key -e SECRET_KEY=secret \\"
    echo "             vprs3bkp:latest backup"
}

# Function to validate required environment variables
validate_env() {
    local missing_vars=()
    
    if [ -z "$DATABASE" ]; then
        missing_vars+=("DATABASE")
    fi
    
    if [ -z "$USERNAME" ]; then
        missing_vars+=("USERNAME")
    fi
    
    if [ -z "$PASSWORD" ]; then
        missing_vars+=("PASSWORD")
    fi
    
    if [ "$STORAGE_TYPE" = "s3" ]; then
        if [ -z "$BUCKET" ]; then
            missing_vars+=("BUCKET")
        fi
        if [ -z "$ACCESS_KEY" ]; then
            missing_vars+=("ACCESS_KEY")
        fi
        if [ -z "$SECRET_KEY" ]; then
            missing_vars+=("SECRET_KEY")
        fi
    fi
    
    if [ ${#missing_vars[@]} -ne 0 ]; then
        echo "❌ Error: Missing required environment variables:"
        for var in "${missing_vars[@]}"; do
            echo "  - $var"
        done
        echo ""
        echo "Use 'docker run vprs3bkp:latest help' for usage information."
        exit 1
    fi
}

# Function to build vprs3bkp command
build_command() {
    local cmd="/usr/local/bin/vprs3bkp $1"
    
    # Add database parameters
    cmd="$cmd --database-type $DATABASE_TYPE"
    cmd="$cmd --database $DATABASE"
    cmd="$cmd --host $HOST"
    cmd="$cmd --port $PORT"
    cmd="$cmd --username $USERNAME"
    cmd="$cmd --password $PASSWORD"
    
    # Add storage parameters
    cmd="$cmd --storage-type $STORAGE_TYPE"
    
    if [ -n "$LOCATION" ]; then
        cmd="$cmd --location $LOCATION"
    fi
    
    if [ "$STORAGE_TYPE" = "s3" ]; then
        cmd="$cmd --bucket $BUCKET"
        cmd="$cmd --region $REGION"
        cmd="$cmd --access-key $ACCESS_KEY"
        cmd="$cmd --secret-key $SECRET_KEY"
        
        if [ -n "$ENDPOINT" ]; then
            cmd="$cmd --endpoint $ENDPOINT"
        fi
    fi
    
    # Add optional parameters
    if [ -n "$BACKUP_NAME" ]; then
        cmd="$cmd --backup-name $BACKUP_NAME"
    fi
    
    if [ "$VERBOSE" = "true" ]; then
        cmd="$cmd --verbose"
    fi
    
    echo "$cmd"
}

# Function to run backup
run_backup() {
    echo "🚀 Starting database backup..."
    echo "Database: $DATABASE_TYPE://$USERNAME@$HOST:$PORT/$DATABASE"
    echo "Storage: $STORAGE_TYPE"
    
    if [ "$STORAGE_TYPE" = "s3" ]; then
        echo "S3 Bucket: $BUCKET"
        echo "S3 Region: $REGION"
        if [ -n "$ENDPOINT" ]; then
            echo "S3 Endpoint: $ENDPOINT"
        fi
    fi
    
    echo ""
    
    validate_env
    
    local cmd=$(build_command "backup")
    echo "Executing: $cmd"
    echo ""
    
    eval "$cmd"
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Backup completed successfully!"
    else
        echo ""
        echo "❌ Backup failed!"
        exit 1
    fi
}

# Function to run restore
run_restore() {
    echo "🔄 Starting database restore..."
    echo "Database: $DATABASE_TYPE://$USERNAME@$HOST:$PORT/$DATABASE"
    echo "Storage: $STORAGE_TYPE"
    echo ""
    
    validate_env
    
    local cmd=$(build_command "restore")
    echo "Executing: $cmd"
    echo ""
    
    eval "$cmd"
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Restore completed successfully!"
    else
        echo ""
        echo "❌ Restore failed!"
        exit 1
    fi
}

# Function to list backups
run_list() {
    echo "📋 Listing available backups..."
    echo ""
    
    local cmd=$(build_command "list")
    echo "Executing: $cmd"
    echo ""
    
    eval "$cmd"
}

# Function to show version
show_version() {
    echo "vprs3bkp Docker Container"
    echo "Built with:"
    /usr/local/bin/vprs3bkp --version
}

# Check if vprs3bkp is available
check_vprs3bkp() {
    if ! command -v /usr/local/bin/vprs3bkp &> /dev/null; then
        echo "❌ Error: vprs3bkp not found at /usr/local/bin/vprs3bkp"
        echo "Available binaries:"
        ls -la /usr/local/bin/ | grep vprs3bkp || echo "No vprs3bkp binaries found"
        exit 1
    fi
}

# Main script logic
check_vprs3bkp

case "${1:-backup}" in
    backup)
        run_backup
        ;;
    restore)
        run_restore
        ;;
    list)
        run_list
        ;;
    version)
        show_version
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo ""
        show_usage
        exit 1
        ;;
esac