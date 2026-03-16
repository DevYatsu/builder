#!/usr/bin/env bash

# builder - Universal install script
# https://github.com/DevYatsu/builder

set -e

# Configuration
REPO="DevYatsu/builder"
BINARY_NAME="builder"
INSTALL_DIR="/usr/local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# OS detection
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)
        PLATFORM="x86_64-unknown-linux-musl"
        ;;
    darwin)
        if [[ "$ARCH" == "arm64" ]]; then
            PLATFORM="aarch64-apple-darwin"
        else
            PLATFORM="x86_64-apple-darwin"
        fi
        ;;
    *)
        error "Unsupported OS: $OS. Please install via Cargo: cargo install builder"
        ;;
esac

info "Detected $OS ($ARCH). Fetching latest release..."

# Get latest release tag
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [[ -z "$LATEST_TAG" ]]; then
    error "Could not fetch latest release tag. Is the repository public?"
fi

info "Latest version is $LATEST_TAG"

# Download URL
FILENAME="${BINARY_NAME}-${PLATFORM}.tar.gz"
URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$FILENAME"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

info "Downloading $BINARY_NAME from $URL..."
curl -L -o "$TMP_DIR/$FILENAME" "$URL" || error "Download failed"

info "Extracting..."
tar -xzf "$TMP_DIR/$FILENAME" -C "$TMP_DIR"

info "Installing to $INSTALL_DIR..."
if [[ -w "$INSTALL_DIR" ]]; then
    mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
else
    info "Requesting sudo permissions to move binary to $INSTALL_DIR"
    sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

success "$BINARY_NAME has been installed successfully!"
echo -e "Try it out by running: ${BLUE}$BINARY_NAME --help${NC}"
