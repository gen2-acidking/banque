#!/bin/bash

set -e

REPO_URL="https://github.com/gen2-acidking/banque"
BINARY_NAME="banque"
ALIAS_NAME="banque"
INSTALL_DIR="$HOME/.local/bin"
SHELL_RC="$HOME/.bashrc"
USE_STATIC=false

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    case $ARCH in
        x86_64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        armv7l) ARCH="armv7" ;;
        *) log_error "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    case $OS in
        linux) OS="linux" ;;
        darwin) OS="macos" ;;
        *) log_error "Unsupported OS: $OS"; exit 1 ;;
    esac
    log_info "Detected platform: $OS-$ARCH"
}

get_latest_release() {
    log_info "Fetching latest release..."
    LATEST_URL="https://api.github.com/repos/gen2-acidking/banque/releases/latest"
    if command -v curl >/dev/null 2>&1; then
        RELEASE_INFO=$(curl -s "$LATEST_URL")
    elif command -v wget >/dev/null 2>&1; then
        RELEASE_INFO=$(wget -qO- "$LATEST_URL")
    else
        log_error "Neither curl nor wget found"
        exit 1
    fi
    
    if [ "$USE_STATIC" = true ]; then
        DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o "https://.*${BINARY_NAME}-linux-musl-${ARCH}.*" | head -1)
    else
        DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o "https://.*${BINARY_NAME}-linux-${ARCH}.*" | head -1)
    fi

    if [ -z "$DOWNLOAD_URL" ]; then
        log_error "Could not find release for $OS-$ARCH"
        exit 1
    fi

    VERSION=$(echo "$RELEASE_INFO" | grep '"tag_name"' | sed 's/.*"tag_name": "*\([^"]*\)".*/\1/')
    log_info "Latest version: $VERSION"
}

install_binary() {
    log_info "Installing binary..."
    mkdir -p "$INSTALL_DIR"
    TEMP_FILE=$(mktemp)

    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$TEMP_FILE" "$DOWNLOAD_URL"
    else
        wget -O "$TEMP_FILE" "$DOWNLOAD_URL"
    fi

    chmod +x "$TEMP_FILE"
    mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
    log_info "Binary installed to $INSTALL_DIR/$BINARY_NAME"
}

setup_shell_integration() {
    log_info "Setting up shell integration..."
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    else
        SHELL_RC="$HOME/.bashrc"
    fi
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
        log_info "Added $INSTALL_DIR to PATH"
    fi
    if ! grep -q "alias $ALIAS_NAME=" "$SHELL_RC" 2>/dev/null; then
        echo "alias $ALIAS_NAME='$INSTALL_DIR/$BINARY_NAME'" >> "$SHELL_RC"
        log_info "Added alias '$ALIAS_NAME'"
    else
        log_warn "Alias '$ALIAS_NAME' already exists"
    fi
}

refresh_environment() {
    log_info "Refreshing environment..."
    export PATH="$INSTALL_DIR:$PATH"
    if [ -f "$SHELL_RC" ]; then
        set +e
        . "$SHELL_RC" >/dev/null 2>&1
        set -e
    fi
    alias $ALIAS_NAME="$INSTALL_DIR/$BINARY_NAME"
    log_info "Environment refreshed"
}

verify_installation() {
    log_info "Verifying installation..."
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        log_info "✓ Binary is executable"
    else
        log_error "Binary is not executable"
        exit 1
    fi
    if "$INSTALL_DIR/$BINARY_NAME" help >/dev/null 2>&1; then
        log_info "✓ Binary runs successfully"
    else
        log_error "Binary failed to run"
        exit 1
    fi
}

usage() {
    echo "Usage: curl -sSL https://raw.githubusercontent.com/gen2-acidking/banque/master/install.sh | bash [-s -- --static]"
    echo "  --static  Install the statically linked binary (default is dynamically linked)"
    exit 1
}

main() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --static)
                USE_STATIC=true
                log_info "Installing statically linked binary"
                shift
                ;;
            *)
                usage
                ;;
        esac
    done

    echo "========================================"
    echo "  Banque Command Tool Installation"
    echo "========================================"
    echo

    detect_platform
    get_latest_release
    install_binary
    setup_shell_integration
    refresh_environment
    verify_installation

    echo
    log_info "Installation completed successfully!"
    echo
    log_info "Ready to use: $ALIAS_NAME"
    log_info "Try: $ALIAS_NAME help"
    echo
}

main "$@"
