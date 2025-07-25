#!/bin/bash
# install.sh - Easy installer for vprs3bkp
set -e

# Configuration
VERSION="2.0.0"  # Update this with your latest version
BINARY_NAME="cli"
INSTALL_DIR="/usr/local/bin"
GITHUB_REPO="vpr-group/vprs3bkp"  # Updated with your actual repo

# Parse command line arguments
INSTALL_FROM_SOURCE=false
INSTALL_DEPENDENCIES=false

print_usage() {
  echo "Usage: $0 [OPTIONS]"
  echo "Options:"
  echo "  --from-source    Install from source code (requires Rust toolchain)"
  echo "  --with-deps      Install dependencies (pg_dump, gzip)"
  echo "  --help           Display this help message and exit"
  echo ""
  echo "Note: The installer now uses statically linked binaries by default for better compatibility."
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
      # Keep this for backward compatibility but ignore it since it's now default
      echo "Note: --musl is now the default behavior"
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
  echo "Building optimized release with vendored OpenSSL..."
  
  # Build for the appropriate target with vendored OpenSSL
  if [ "$OS" = "macos" ]; then
    ARCH=$(uname -m)
    if [ "$ARCH" = "arm64" ]; then
      TARGET="aarch64-apple-darwin"
    else
      TARGET="x86_64-apple-darwin"
    fi
    rustup target add $TARGET
    cd cli
    cargo build --release --target $TARGET --features vendored-openssl
    # Install the binary
    $SUDO_CMD cp ../target/$TARGET/release/$BINARY_NAME $INSTALL_DIR/
  else
    # Linux - use musl for static linking
    rustup target add x86_64-unknown-linux-musl
    # Install musl tools if needed
    if command -v apt-get &> /dev/null; then
      sudo apt-get update && sudo apt-get install -y musl-tools musl-dev
    elif command -v dnf &> /dev/null; then
      sudo dnf install -y musl-gcc musl-devel
    elif command -v yum &> /dev/null; then
      sudo yum install -y musl-gcc musl-devel
    fi
    cd cli
    cargo build --release --target x86_64-unknown-linux-musl --features vendored-openssl
    # Install the binary
    $SUDO_CMD cp ../target/x86_64-unknown-linux-musl/release/$BINARY_NAME $INSTALL_DIR/
  fi
  
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
    # Linux - always use musl builds for better compatibility
    if [ "$ARCH" = "x86_64" ]; then
      ARTIFACT_NAME="linux-x86_64-musl"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
      echo "ARM64 Linux builds not yet available. Please use --from-source instead."
      echo "Or install on an x86_64 system."
      exit 1
    else
      echo "Unsupported architecture: $ARCH"
      echo "Please install from source with --from-source"
      exit 1
    fi
  fi
  
  # New GitHub release asset URL format
  BINARY_URL="https://github.com/$GITHUB_REPO/releases/latest/download/vprs3bkp-$ARTIFACT_NAME"
  
  echo "Downloading statically linked binary from $BINARY_URL..."
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
  echo "Binary info:"
  if [ "$OS" = "linux" ]; then
    echo "  • Statically linked (no external dependencies)"
    echo "  • Works on any Linux distribution"
  else
    echo "  • Optimized for $OS"
  fi
  echo "  • Version: $VERSION"
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