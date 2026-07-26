#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

REPO="Ivan23BG/latex_handler"
ARCHIVE_URL="https://github.com/$REPO/releases/latest/download/latex_handler-linux.tar.gz"

BIN_DIR="$HOME/.local/bin"
EXE_PATH="$BIN_DIR/latex_handler"

echo "Installing latex_handler..."

# Create the bin directory if it doesn't exist
mkdir -p "$BIN_DIR"

echo "Downloading and extracting the latest release..."
if ! curl -sSL --fail "$ARCHIVE_URL" | tar -xz -C "$BIN_DIR" latex_handler; then
    echo "  [Error] Failed to download or extract the binary. Please check if a release exists."
    exit 1
fi

# Ensure executable permissions
chmod +x "$EXE_PATH"

echo "[Success] Binary installed to $EXE_PATH"

echo "Running initial setup to fetch LaTeX templates..."
"$EXE_PATH" update

echo "Installation complete!"

# Warn if ~/.local/bin is not in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "[Warning] $BIN_DIR is not in your PATH."
    echo "   Please add 'export PATH=\"\$HOME/.local/bin:\$PATH\"' to your shell config (~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish)."
fi