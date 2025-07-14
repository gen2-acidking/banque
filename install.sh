#!/bin/bash
################
set -e

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

install_binary() {
    log_info "Installing binary..."
    mkdir -p "$INSTALL_DIR"
    
    if [ "$USE_STATIC" = true ]; then
        DOWNLOAD_URL="https://github.com/gen2-acidking/banque/releases/download/v1.0.0/banque-linux-musl-x86_64"
        log_info "Downloading static binary..."
    else
        DOWNLOAD_URL="https://github.com/gen2-acidking/banque/releases/download/v1.0.0/banque-linux-x86_64"
        log_info "Downloading dynamic binary..."
    fi
    
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$INSTALL_DIR/$BINARY_NAME" "$DOWNLOAD_URL"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$INSTALL_DIR/$BINARY_NAME" "$DOWNLOAD_URL"
    else
        log_error "Neither curl nor wget found"
        exit 1
    fi
    
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
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

main() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --static)
                USE_STATIC=true
                log_info "Installing statically linked binary"
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    echo "========================================"
    echo "  Banque Installation"
    echo "========================================"
    echo

    install_binary
    setup_shell_integration
    refresh_environment
    verify_installation

    echo
    log_info "Installation completed successfully!"
    echo
    log_info "Ready to use: $ALIAS_NAME"
    log_info "Try: $ALIAS_NAME help"
    log_info "Refresh the terminal fucking subshell can't. Make a pull request if you want to fix it"
    echos
}

main "$@"
