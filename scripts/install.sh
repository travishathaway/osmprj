#!/usr/bin/env bash
set -euo pipefail

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) PLATFORM="linux-64" ;;
      aarch64) PLATFORM="linux-aarch64" ;;
      *)
        echo "Unsupported architecture: $ARCH on Linux. Only x86_64 and aarch64 is supported." >&2
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      arm64) PLATFORM="osx-arm64" ;;
      *)
        echo "Unsupported architecture: $ARCH on macOS. Only arm64 is supported." >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported operating system: $OS. Only Linux (linux-64) and macOS (osx-arm64) are supported." >&2
    exit 1
    ;;
esac

INSTALL_DIR="${OSMPRJ_INSTALL_DIR:-$HOME/.local/osmprj}"
RELEASE_BASE="https://github.com/travishathaway/osmprj/releases/latest/download"

# Download platform-specific installer
echo "Downloading osmprj installer for $PLATFORM..."
curl -fsSL "$RELEASE_BASE/osmprj-${PLATFORM}-installer.sh" -o /tmp/osmprj-installer.sh
chmod +x /tmp/osmprj-installer.sh

# Run the installer
echo "Installing osmprj to $INSTALL_DIR..."
/tmp/osmprj-installer.sh --output-directory "$INSTALL_DIR"

# Detect shell rc file
if [[ "$SHELL" == *zsh* ]]; then
  RC_FILE="$HOME/.zshrc"
elif [[ "$SHELL" == *bash* ]]; then
  RC_FILE="$HOME/.bashrc"
else
  RC_FILE="$HOME/.profile"
fi

# Idempotent rc file patching
if ! grep -q "# osmprj" "$RC_FILE" 2>/dev/null; then
  {
    echo ""
    echo "# osmprj"
    echo "export PATH=\"$INSTALL_DIR/env/bin:\$PATH\""
    echo "export OSMPRJ_THEME_PATH=\"$INSTALL_DIR/env/share/osmprj/themes/\""
  } >> "$RC_FILE"
fi

# Cleanup
rm /tmp/osmprj-installer.sh

# Success message
echo ""
echo "osmprj has been installed to $INSTALL_DIR"
echo ""
echo "Restart your shell or run:"
echo "  source $RC_FILE"
