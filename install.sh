#!/bin/bash
# install.sh - Easy installer for db-backup-cli

set -e

# Configuration
VERSION="0.1.0"
BINARY_NAME="db-backup-cli"
INSTALL_DIR="/usr/local/bin"
GITHUB_REPO="yourusername/db-backup-cli" # Replace with your actual repo

# Parse command line arguments
INSTALL_FROM_SOURCE=false
INSTALL_DEPENDENCIES=false

print_usage() {
  echo "Usage: $0 [OPTIONS]"
  echo "Options:"
  echo "  --from-source       Install from source code (requires Rust toolchain)"
  echo "  --with-deps         Install dependencies (pg_dump, mysqldump, gzip)"
  echo "  --help              Display this help message and exit"
}

for arg in "$@"; do
  case $arg in
    --from-source)
      INSTALL_FROM_SOURCE=true
      shift
      ;;
    --with-deps)
      INSTALL_DEPENDENCIES=true
      shift
      ;;
    --help)
      print_usage
      exit 0
      ;;
    *)
      echo "Unknown option: $arg"
      print_usage
      exit 1
      ;;
  esac
done

# Detect OS
if [ -f /etc/os-release ]; then
  . /etc/os-release
  OS=$ID
else
  echo "Cannot detect operating system"
  exit 1
fi

# Install dependencies if requested
if [ "$INSTALL_DEPENDENCIES" = true ]; then
  echo "Installing dependencies..."
  
  case $OS in
    ubuntu|debian)
      apt-get update
      apt-get install -y postgresql-client mysql-client gzip curl
      ;;
    centos|rhel|fedora)
      if command -v dnf &> /dev/null; then
        dnf install -y postgresql mysql gzip curl
      else
        yum install -y postgresql mysql gzip curl
      fi
      ;;
    *)
      echo "Unsupported OS for automatic dependency installation"
      echo "Please install postgresql-client, mysql-client and gzip manually"
      ;;
  esac
fi

# Create installation directory if it doesn't exist
mkdir -p $INSTALL_DIR

# Install from source or binary
if [ "$INSTALL_FROM_SOURCE" = true ]; then
  echo "Installing from source..."
  
  # Check if Rust is installed
  if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
  fi
  
  # Create a temporary directory
  TMP_DIR=$(mktemp -d)
  cd $TMP_DIR
  
  # Clone the repository or download and extract source tarball
  echo "Downloading source code..."
  if command -v git &> /dev/null; then
    git clone https://github.com/$GITHUB_REPO .
  else
    curl -L "https://github.com/$GITHUB_REPO/archive/v$VERSION.tar.gz" | tar xz --strip-components=1
  fi
  
  # Build the project
  echo "Building project..."
  cargo build --release
  
  # Install the binary
  cp target/release/$BINARY_NAME $INSTALL_DIR/
  
  # Clean up
  cd - > /dev/null
  rm -rf $TMP_DIR
else
  # Download pre-built binary based on architecture
  ARCH=$(uname -m)
  case $ARCH in
    x86_64)
      ARCH_NAME="x86_64"
      ;;
    aarch64|arm64)
      ARCH_NAME="aarch64"
      ;;
    *)
      echo "Unsupported architecture: $ARCH"
      echo "Please install from source with --from-source"
      exit 1
      ;;
  esac
  
  BINARY_URL="https://github.com/$GITHUB_REPO/releases/download/v$VERSION/$BINARY_NAME-$VERSION-$OS-$ARCH_NAME.tar.gz"
  
  echo "Downloading pre-built binary from $BINARY_URL..."
  curl -L "$BINARY_URL" | tar xz -C $INSTALL_DIR
fi

# Make binary executable
chmod +x $INSTALL_DIR/$BINARY_NAME

# Verify installation
if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
  echo "Installation successful!"
  echo "The $BINARY_NAME tool is now available at $INSTALL_DIR/$BINARY_NAME"
  echo ""
  echo "Example usage:"
  echo "$BINARY_NAME --bucket my-backup-bucket --region us-west-2 postgres --database mydb --username dbuser"
else
  echo "Installation failed."
  exit 1
fi