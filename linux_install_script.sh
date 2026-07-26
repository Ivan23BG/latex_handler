#!/bin/bash

#STREAMING_CHUNK:Configuring fail-safes and variables...

#Exit immediately if a command exits with a non-zero status

set -e

#TODO: Replace with your actual GitHub username and repository name

REPO="Ivan23BG/latex_handler"

#We expect the GitHub Release to have a file named 'latex_handler-linux'

BINARY_URL="https://github.com/$REPO/releases/latest/download/latex_handler-linux"

#STREAMING_CHUNK:Setting up local directories...

#~/.local/bin is the standard location for user-specific binaries on Linux

BIN_DIR="$HOME/.local/bin"
EXE_PATH="$BIN_DIR/latex_handler"

echo "🚀 Installing latex_handler..."*

#Create the bin directory if it doesn't exist

mkdir -p "$BIN_DIR"

#STREAMING_CHUNK:Downloading and installing binary...

echo "📥 Downloading the latest release from GitHub..."
if ! curl -sSL --fail "$BINARY_URL" -o "$EXE_PATH"; then
echo "❌ Failed to download the binary. Please check the URL and ensure a release exists."
exit 1
fi

#Make the downloaded file executable

chmod +x "$EXE_PATH"

echo "✅ Binary installed to $EXE_PATH"

#STREAMING_CHUNK:Running initial setup and verifying PATH...

echo "🔄 Running initial setup to fetch LaTeX templates..."
"$EXE_PATH" update

echo "🎉 Installation complete!"

#Check if ~/.local/bin is in the user's PATH, warn them if not

if [[ ":$PATH:" != ":$BIN_DIR:" ]]; then
echo "⚠️  NOTE: $BIN_DIR is not in your PATH."
echo "   Please add 'export PATH="$HOME/.local/bin:$PATH"' to your ~/.bashrc or ~/.zshrc to use the command anywhere."
fi


# one liner: curl -sSL [https://raw.githubusercontent.com/YourUsername/YourRepo/main/install.sh](https://raw.githubusercontent.com/YourUsername/YourRepo/main/install.sh) | bash