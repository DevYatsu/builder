#!/usr/bin/env bash

# builder - Uninstall script
# https://github.com/DevYatsu/builder

set -e

BINARY_NAME="builder"
PATHS=("/usr/local/bin/$BINARY_NAME" "$HOME/.local/bin/$BINARY_NAME")

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

UNINSTALLED=false

for path in "${PATHS[@]}"; do
    if [[ -f "$path" ]]; then
        info "Found $BINARY_NAME at $path. Removing..."
        if [[ -w "$path" ]]; then
            rm "$path"
        else
            info "Requesting sudo permissions to remove binary from $path"
            sudo rm "$path"
        fi
        UNINSTALLED=true
    fi
done

if [ "$UNINSTALLED" = true ]; then
    success "$BINARY_NAME was successfully uninstalled."
else
    info "Could not find $BINARY_NAME in common installation paths."
fi
