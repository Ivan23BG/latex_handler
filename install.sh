#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

REPO="Ivan23BG/latex_handler"
ARCHIVE_URL="https://github.com/$REPO/releases/latest/download/latex_handler-linux.tar.gz"

BIN_DIR="$HOME/.local/bin"
EXE_PATH="$BIN_DIR/latex_handler"

echo "Installing latex_handler..."

# Create target directory if it doesn't exist
mkdir -p "$BIN_DIR"

echo "Downloading latest release binary..."
if ! curl -sSL --fail "$ARCHIVE_URL" | tar -xz -C "$BIN_DIR" latex_handler; then
    echo "  [Error] Failed to download or extract binary. Verify that a release asset exists at $ARCHIVE_URL"
    exit 1
fi

chmod +x "$EXE_PATH"
echo " [Success] Binary installed to $EXE_PATH"

# Check PATH variable for bash/zsh/fish
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo ""
    echo "  [Note] $BIN_DIR is not in your current PATH."
    echo "   For bash/zsh, add: export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo "   For fish, run:     fish_add_path ~/.local/bin"
fi

echo " [Info] Installation complete!"