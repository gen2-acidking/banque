#!/bin/bash

set -e

BINARY_NAME="banque"
INSTALL_DIR="$HOME/.local/bin"
SHELL_RC="$HOME/.bashrc"
ALIAS_LINE="alias $BINARY_NAME="
PATH_LINE="export PATH=\"$INSTALL_DIR:\$PATH\""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

detect_shell_rc() {
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    else
        SHELL_RC="$HOME/.bashrc"
    fi
}

remove_binary() {
    if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        rm -f "$INSTALL_DIR/$BINARY_NAME"
        log_info "Removed binary from $INSTALL_DIR/$BINARY_NAME"
    else
        log_warn "Binary not found in $INSTALL_DIR"
    fi
}

remove_alias_and_path() {
    if [ -f "$SHELL_RC" ]; then
        sed -i "/$ALIAS_LINE/d" "$SHELL_RC"
        sed -i "/$PATH_LINE/d" "$SHELL_RC"
        log_info "Removed alias and PATH export from $SHELL_RC"
    else
        log_warn "Shell config file not found"
    fi
}

main() {
    echo "========================================"
    echo "  Banque Uninstallation"
    echo "========================================"
    echo

    detect_shell_rc
    remove_binary
    remove_alias_and_path

    log_info "Uninstallation complete. You may want to restart your shell."
}

main
