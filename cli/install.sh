#!/bin/bash
# install.sh - Easy installer for vprs3bkp
set -e

# Configuration
VERSION="0.3.2"  # Update this with your latest version
BINARY_NAME="cli"
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
  echo "  --with-deps      Install dependencies (pg_dump, gzip)"
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
OS="unknown"
if [ "$(uname)" == "Darwin" ]; then
  OS="macos"
elif [ -f /etc/os-release ]; then
  . /etc/os-release
  OS=$ID
else
  echo "Warning: Cannot precisely detect operating system, assuming Linux"
  OS="linux"
fi

echo "Detected OS: $OS"

# Install dependencies if requested
if [ "$INSTALL_DEPENDENCIES" = true ]; then
  echo "Installing dependencies..."
  case $OS in
    ubuntu|debian|linuxmint)
      echo "Updating package repositories (ignoring errors)..."
      sudo apt-get update 2>/dev/null || true
      echo "Installing required packages..."
      sudo apt-get install -y --no-install-recommends postgresql-client gzip curl
      if [ $? -ne 0 ]; then
        echo "Warning: Some packages may not have installed correctly."
        echo "Continuing with installation anyway..."
      fi
      ;;
    centos|rhel|fedora)
      if command -v dnf &> /dev/null; then
        sudo dnf install -y postgresql gzip curl
      else
        sudo yum install -y postgresql gzip curl
      fi
      ;;
    macos)
      if command -v brew &> /dev/null; then
        echo "Installing dependencies with Homebrew..."
        brew install postgresql gzip curl
      else
        echo "Homebrew not found. Please install Homebrew first:"
        echo "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        echo "Then install the dependencies with: brew install postgresql"
        echo "Continuing with installation anyway..."
      fi
      ;;
    *)
      echo "Unsupported OS for automatic dependency installation"
      echo "Please install postgresql-client and gzip manually"
      ;;
  esac
fi

# Determine if we need sudo (usually not on macOS)
SUDO_CMD="sudo"
if [ "$OS" = "macos" ] && [ -w "$INSTALL_DIR" ]; then
  # If user can write to the install directory on macOS, don't use sudo
  SUDO_CMD=""
fi

# Create installation directory if it doesn't exist
$SUDO_CMD mkdir -p $INSTALL_DIR

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

  # Build the project with optimizations for small binary size
  echo "Building optimized release..."
  cat > Cargo.toml <<EOF
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
debug = false
EOF

  cargo build --release

  # Install the binary
  $SUDO_CMD cp target/release/$BINARY_NAME $INSTALL_DIR/
  $SUDO_CMD mv $INSTALL_DIR/$BINARY_NAME $INSTALL_DIR/vprs3bkp

  # Clean up
  cd - > /dev/null
  rm -rf $TMP_DIR
else
  # Download pre-built binary based on architecture and OS
  ARCH=$(uname -m)
  
  if [ "$OS" = "macos" ]; then
    if [ "$ARCH" = "arm64" ]; then
      ARTIFACT_NAME="macos-silicon"
    else
      ARTIFACT_NAME="macos-intel"
    fi
  else
    # Linux
    if [ "$ARCH" = "x86_64" ]; then
      if [ "$USE_MUSL" = true ]; then
        ARTIFACT_NAME="linux-x86_64-musl"
      else
        ARTIFACT_NAME="linux-x86_64"
      fi
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
      if [ "$USE_MUSL" = true ]; then
        ARTIFACT_NAME="linux-aarch64-musl"
      else
        ARTIFACT_NAME="linux-aarch64"
      fi
    else
      echo "Unsupported architecture: $ARCH"
      echo "Please install from source with --from-source"
      exit 1
    fi
  fi
  
  # New GitHub release asset URL format
  BINARY_URL="https://github.com/$GITHUB_REPO/releases/latest/download/vprs3bkp-$ARTIFACT_NAME"
  
  echo "Downloading pre-built binary from $BINARY_URL..."
  # Create a temporary directory
  TMP_DIR=$(mktemp -d)
  
  # Download the binary to the temporary location
  if curl -L "$BINARY_URL" -o "$TMP_DIR/vprs3bkp"; then
    # Move to final location
    $SUDO_CMD mv "$TMP_DIR/vprs3bkp" "$INSTALL_DIR/vprs3bkp"
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
$SUDO_CMD chmod +x $INSTALL_DIR/vprs3bkp

# Verify installation
if [ -x "$INSTALL_DIR/vprs3bkp" ]; then
  echo "Installation successful!"
  echo "The vprs3bkp tool is now available at $INSTALL_DIR/vprs3bkp"
  echo ""
  echo "Example usage:"
  echo "vprs3bkp backup --database mydb --host localhost --username postgres \\"
  echo "  --storage-type s3 --bucket my-backup-bucket --region us-west-2"
  echo ""
  echo "For more options, run: vprs3bkp --help"
else
  echo "Installation failed."
  exit 1
fi