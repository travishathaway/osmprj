#!/usr/bin/env bash
set -euo pipefail

# Mask username:password in URLs before printing to stdout/stderr
mask_credentials() {
    echo "$1" | sed -E 's|://[^:@/]+:[^@/]+@|://***:***@|g'
}

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"
# IS_MSYS=false  # future: detect Msys/Windows here

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) PLATFORM="linux-64" ;;
      aarch64) PLATFORM="linux-aarch64" ;;
      *)
        echo "Unsupported architecture: $ARCH on Linux. Only x86_64 and aarch64 are supported." >&2
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
    # future: add Windows support by detecting IS_MSYS=true above
    exit 1
    ;;
esac

VERSION="${OSMPRJ_VERSION:-latest}"
INSTALL_DIR="${OSMPRJ_INSTALL_DIR:-$HOME/.local/osmprj}"
case "$INSTALL_DIR" in
    '~' | '~'/*) INSTALL_DIR="${HOME-}${INSTALL_DIR#\~}" ;; # expand tilde
esac
REPOURL="https://github.com/travishathaway/osmprj"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="${OSMPRJ_DOWNLOAD_URL:-${REPOURL}/releases/latest/download/osmprj-${PLATFORM}-installer.sh}"
else
    # Prepend 'v' if missing
    DOWNLOAD_URL="${OSMPRJ_DOWNLOAD_URL:-${REPOURL}/releases/download/v${VERSION#v}/osmprj-${PLATFORM}-installer.sh}"
fi

# Detect available download tool
HAVE_CURL=false
HAVE_CURL_8_8_0=false
if hash curl 2>/dev/null; then
    # curl 8.8.0 is broken for --write-out: https://github.com/curl/curl/issues/13845
    if [ "$(curl --version | (IFS=' ' read -r _ v _; printf %s "${v-}"))" = "8.8.0" ]; then
        HAVE_CURL_8_8_0=true
    else
        HAVE_CURL=true
    fi
fi

HAVE_WGET=true
hash wget 2>/dev/null || HAVE_WGET=false

if ! $HAVE_CURL && ! $HAVE_WGET; then
    echo "error: you need either 'curl' or 'wget' installed for this script." >&2
    if $HAVE_CURL_8_8_0; then
        echo "error: curl 8.8.0 is known to be broken, please use a different version." >&2
    fi
    exit 1
fi

# Suppress progress bars when stdout is not a terminal (e.g. CI, piped output)
if [ ! -t 1 ]; then
    CURL_OPTIONS="--silent"
    WGET_OPTIONS="--no-verbose"
else
    CURL_OPTIONS=""
    WGET_OPTIONS="--show-progress"
fi

# Use .netrc for authentication if available
if [ -n "${NETRC:-}" ]; then
    CURL_OPTIONS="$CURL_OPTIONS --netrc-file $NETRC"
    WGET_OPTIONS="$WGET_OPTIONS --netrc-file=$NETRC"
elif [ -f "$HOME/.netrc" ]; then
    CURL_OPTIONS="$CURL_OPTIONS --netrc"
    WGET_OPTIONS="$WGET_OPTIONS --netrc"
fi

# Download platform-specific installer
printf "This script will download and install osmprj (%s) for you.\nDownloading from: %s\n" "$VERSION" "$(mask_credentials "$DOWNLOAD_URL")"
INSTALLER_SCRIPT="$(mktemp "${TMPDIR:-/tmp}/osmprj-installer.XXXXXX.sh")"
trap 'rm -f "$INSTALLER_SCRIPT"' EXIT

if $HAVE_CURL; then
    CURL_ERR=0
    # shellcheck disable=SC2086
    HTTP_CODE="$(curl -SL $CURL_OPTIONS "$DOWNLOAD_URL" --output "$INSTALLER_SCRIPT" --write-out "%{http_code}")" || CURL_ERR=$?
    case "$CURL_ERR" in
    35 | 53 | 54 | 59 | 66 | 77)
        if ! $HAVE_WGET; then
            echo "error: curl encountered an SSL error ($CURL_ERR) downloading '$(mask_credentials "$DOWNLOAD_URL")'." >&2
            exit 1
        fi
        HAVE_CURL=false # fall through to wget
        ;;
    0)
        if [ "${HTTP_CODE}" -eq 401 ]; then
            echo "error: authentication failed downloading '$(mask_credentials "$DOWNLOAD_URL")'." >&2
            echo "       Check your .netrc file or the NETRC environment variable." >&2
            exit 1
        elif [ "${HTTP_CODE}" -lt 200 ] || [ "${HTTP_CODE}" -gt 299 ]; then
            echo "error: '$(mask_credentials "$DOWNLOAD_URL")' is not available (HTTP ${HTTP_CODE})." >&2
            exit 1
        fi
        HAVE_WGET=false # success, skip wget
        ;;
    *)
        echo "error: curl failed with error $CURL_ERR downloading '$(mask_credentials "$DOWNLOAD_URL")'." >&2
        exit 1
        ;;
    esac
fi

if $HAVE_WGET; then
    # shellcheck disable=SC2086
    if ! wget $WGET_OPTIONS --output-document="$INSTALLER_SCRIPT" "$DOWNLOAD_URL"; then
        echo "error: '$(mask_credentials "$DOWNLOAD_URL")' is not available." >&2
        exit 1
    fi
fi

chmod +x "$INSTALLER_SCRIPT"

# Guard against a silent failure where the file is created but empty
if [ ! -s "$INSTALLER_SCRIPT" ]; then
    echo "error: downloaded file is empty. Check that TMPDIR is writable, or set TMPDIR to a directory with write permissions." >&2
    exit 1
fi

# Run the installer
echo "Installing osmprj to $INSTALL_DIR..."
"$INSTALLER_SCRIPT" --output-directory "$INSTALL_DIR"

echo ""
echo "osmprj has been installed to '$INSTALL_DIR'."
echo ""

# Detect shell rc file and patch PATH (suppress with OSMPRJ_NO_PATH_UPDATE)
if [ -n "${OSMPRJ_NO_PATH_UPDATE:-}" ]; then
    echo "No PATH update because OSMPRJ_NO_PATH_UPDATE is set."
else
    update_shell() {
        FILE="$1"
        LINE="$2"
        if [ ! -f "$FILE" ]; then
            touch "$FILE"
        fi
        if ! grep -Fxq "$LINE" "$FILE"; then
            echo "Updating '$FILE'"
            echo >> "$FILE"
            echo "$LINE" >> "$FILE"
            echo "Please restart or source your shell."
        fi
    }

    case "$(basename "${SHELL-}")" in
    zsh)
        update_shell "$HOME/.zshrc" "export PATH=\"$INSTALL_DIR/env/bin:\$PATH\""
        update_shell "$HOME/.zshrc" "export OSMPRJ_THEME_PATH=\"$INSTALL_DIR/env/share/osmprj/themes/\""
        RC_FILE="$HOME/.zshrc"
        ;;
    bash)
        update_shell "$HOME/.bashrc" "export PATH=\"$INSTALL_DIR/env/bin:\$PATH\""
        update_shell "$HOME/.bashrc" "export OSMPRJ_THEME_PATH=\"$INSTALL_DIR/env/share/osmprj/themes/\""
        RC_FILE="$HOME/.bashrc"
        ;;
    fish)
        update_shell "$HOME/.config/fish/config.fish" "set -gx PATH \"$INSTALL_DIR/env/bin\" \$PATH"
        update_shell "$HOME/.config/fish/config.fish" "set -gx OSMPRJ_THEME_PATH \"$INSTALL_DIR/env/share/osmprj/themes/\""
        RC_FILE="$HOME/.config/fish/config.fish"
        ;;
    tcsh)
        update_shell "$HOME/.tcshrc" "set path = ( $INSTALL_DIR/env/bin \$path )"
        RC_FILE="$HOME/.tcshrc"
        ;;
    '')
        echo "warn: could not detect shell type." >&2
        echo "      Please add '$INSTALL_DIR/env/bin' to your \$PATH manually." >&2
        RC_FILE=""
        ;;
    *)
        echo "warn: could not update shell '$(basename "$SHELL")'." >&2
        echo "      Please add '$INSTALL_DIR/env/bin' to your \$PATH manually." >&2
        RC_FILE=""
        ;;
    esac

    if [ -n "${RC_FILE:-}" ]; then
        echo ""
        echo "Restart your shell or run:"
        echo "  source $RC_FILE"
    fi
fi
