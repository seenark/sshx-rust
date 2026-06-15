#!/usr/bin/env bash
set -euo pipefail

REPO="seenark/sshx-rust"
BINARY="sshx"
INSTALL_DIR="${SSHX_INSTALL_DIR:-/usr/local/bin}"

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS-$ARCH" in
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
  Linux-x86_64)  TARGET="x86_64-unknown-linux-musl" ;;
  Linux-aarch64) TARGET="aarch64-unknown-linux-musl" ;;
  *)
    echo "✗ Unsupported platform: $OS-$ARCH"
    exit 1
    ;;
esac

# Get latest version tag
VERSION=$(curl -sf "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' \
  | sed -E 's/.*"v([^"]+)".*/\1/')

if [[ -z "$VERSION" ]]; then
  echo "✗ Could not determine latest sshx release from https://api.github.com/repos/$REPO/releases/latest"
  exit 1
fi

echo "→ Installing sshx v$VERSION for $TARGET"

URL="https://github.com/$REPO/releases/download/v$VERSION/sshx-v$VERSION-$TARGET.tar.gz"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -sfL "$URL" | tar xz -C "$TMP"

chmod +x "$TMP/$BINARY"

if [[ -w "$INSTALL_DIR" ]]; then
  mv "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
else
  sudo mv "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
fi

echo "✓ sshx installed to $INSTALL_DIR/$BINARY"
echo "  Run: sshx --help"
