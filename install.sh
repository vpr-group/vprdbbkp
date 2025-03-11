#!/bin/bash
# install.sh - Easy installer for vprs3bkp
set -e

# Configuration
VERSION="0.1.1"  # Update this with your latest version
BINARY_NAME="vprs3bkp"
INSTALL_DIR="/usr/local/bin"
GITHUB_REPO="vpr-group/vprs3bkp"  # Updated with your actual repo

# Parse command line arguments
INSTALL_FROM_SOURCE=false
INSTALL_DEPENDENCIES=false
USE_MUSL=false

print_usage() {
  echo "Usage: $0 [OPTIONS]"
  echo "Options:"
  echo "  --from-source    Install from source code (requires Rust toolchain)"
  echo "  --with-deps      Install dependencies (pg_dump, mysqldump, gzip)"
  echo "  --musl           Use statically linked MUSL build (better compatibility)"
  echo "  --help           Display this help message and exit"
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
    --musl)
      USE_MUSL=true
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
    ubuntu|debian|linuxmint)
      echo "Updating package repositories (ignoring errors)..."
      sudo apt-get update 2>/dev/null || true
      echo "Installing required packages..."
      sudo apt-get install -y --no-install-recommends postgresql-client mysql-client gzip curl
      if [ $? -ne 0 ]; then
        echo "Warning: Some packages may not have installed correctly."
        echo "Continuing with installation anyway..."
      fi
      ;;
    centos|rhel|fedora)
      if command -v dnf &> /dev/null; then
        sudo dnf install -y postgresql mysql gzip curl
      else
        sudo yum install -y postgresql mysql gzip curl
      fi
      ;;
    *)
      echo "Unsupported OS for automatic dependency installation"
      echo "Please install postgresql-client, mysql-client and gzip manually"
      ;;
  esac
fi

# Create installation directory if it doesn't exist
sudo mkdir -p $INSTALL_DIR

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
  sudo cp target/release/$BINARY_NAME $INSTALL_DIR/

  # Clean up
  cd - > /dev/null
  rm -rf $TMP_DIR
else
  # Download pre-built binary based on architecture
  ARCH=$(uname -m)
  
  case $ARCH in
    x86_64)
      if [ "$USE_MUSL" = true ]; then
        ARTIFACT_NAME="linux-x86_64-musl"
      else
        ARTIFACT_NAME="linux-x86_64"
      fi
      ;;
    aarch64|arm64)
      ARTIFACT_NAME="macos-silicon"
      ;;
    *)
      echo "Unsupported architecture: $ARCH"
      echo "Please install from source with --from-source"
      exit 1
      ;;
  esac
  
  # New GitHub release asset URL format
  BINARY_URL="https://github.com/$GITHUB_REPO/releases/latest/download/$BINARY_NAME-$ARTIFACT_NAME"
  
  echo "Downloading pre-built binary from $BINARY_URL..."
  # Create a temporary directory
  TMP_DIR=$(mktemp -d)
  
  # Download the binary to the temporary location
  if curl -L "$BINARY_URL" -o "$TMP_DIR/$BINARY_NAME"; then
    # Move to final location
    sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    # Clean up
    rm -rf "$TMP_DIR"
  else
    echo "Error: Failed to download from $BINARY_URL"
    echo "Please check if the release exists and is publicly accessible."
    rm -rf "$TMP_DIR"
    exit 1
  fi
fi

# Make binary executable
sudo chmod +x $INSTALL_DIR/$BINARY_NAME

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