#!/usr/bin/env sh
# mcpobs installer
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/vnmoorthy/mcpobservatory/main/scripts/install.sh | sh
#
# Detects platform, downloads the matching tarball from the latest GitHub
# release, verifies sha256, and installs the `mcpobs` binary into
# ~/.local/bin (or /usr/local/bin if writable).

set -eu

REPO="${MCPOBS_REPO:-vnmoorthy/mcpobservatory}"
VERSION="${MCPOBS_VERSION:-latest}"

OS=""
case "$(uname -s)" in
  Darwin) OS="apple-darwin" ;;
  Linux)  OS="unknown-linux-gnu" ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

ARCH=""
case "$(uname -m)" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "unsupported arch: $(uname -m)" >&2; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"

if [ "$VERSION" = "latest" ]; then
  VERSION="$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": *"v\([^"]*\)".*/\1/p' | head -1)"
  if [ -z "$VERSION" ]; then
    echo "failed to resolve latest version from ${REPO}; set MCPOBS_VERSION=x.y.z to override" >&2
    exit 1
  fi
fi

NAME="mcpobs-${VERSION}-${TARGET}"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${NAME}.tar.gz"
SHA_URL="${URL}.sha256"

# Pick install dir. Prefer ~/.local/bin (Linux convention, no sudo needed).
INSTALL_DIR="${MCPOBS_INSTALL_DIR:-$HOME/.local/bin}"
if [ "$INSTALL_DIR" = "$HOME/.local/bin" ] && [ -w "/usr/local/bin" ]; then
  INSTALL_DIR="/usr/local/bin"
fi

mkdir -p "$INSTALL_DIR"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "downloading ${URL}"
curl -fSLo "$TMP/${NAME}.tar.gz" "$URL"

echo "verifying sha256"
if curl -fSLo "$TMP/${NAME}.tar.gz.sha256" "$SHA_URL"; then
  EXPECTED="$(awk '{print $1}' "$TMP/${NAME}.tar.gz.sha256")"
  if command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "$TMP/${NAME}.tar.gz" | awk '{print $1}')"
  else
    ACTUAL="$(sha256sum "$TMP/${NAME}.tar.gz" | awk '{print $1}')"
  fi
  if [ "$EXPECTED" != "$ACTUAL" ]; then
    echo "sha256 mismatch! expected=$EXPECTED actual=$ACTUAL" >&2
    exit 1
  fi
else
  echo "warning: no sha256 available at $SHA_URL; continuing without verification" >&2
fi

tar -C "$TMP" -xzf "$TMP/${NAME}.tar.gz"
mv "$TMP/${NAME}/mcpobs" "$INSTALL_DIR/mcpobs"
chmod +x "$INSTALL_DIR/mcpobs"

echo "installed: $INSTALL_DIR/mcpobs"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo
    echo "WARNING: $INSTALL_DIR is not on your PATH."
    echo "Add this to your shell rc:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

echo
echo "next: run \`mcpobs init\` then \`mcpobs start\`."
