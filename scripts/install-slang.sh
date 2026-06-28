#!/usr/bin/env bash
#
# Install the Slang shader compiler (slangc) for Linux x86_64.
#
# Downloads a pinned standalone Slang release from GitHub and extracts it to
# ~/.local/slang, then symlinks slangc into ~/.local/bin (usually already on PATH).
#
# Usage:
#   ./scripts/install-slang.sh            # install pinned version
#   SLANG_VERSION=2026.12 ./scripts/install-slang.sh   # override version
#
set -euo pipefail

SLANG_VERSION="${SLANG_VERSION:-2026.12}"
ARCH="linux-x86_64"
ASSET="slang-${SLANG_VERSION}-${ARCH}.tar.gz"
URL="https://github.com/shader-slang/slang/releases/download/v${SLANG_VERSION}/${ASSET}"

INSTALL_DIR="${HOME}/.local/slang/${SLANG_VERSION}"
BIN_DIR="${HOME}/.local/bin"

echo "Installing Slang ${SLANG_VERSION} -> ${INSTALL_DIR}"

mkdir -p "${INSTALL_DIR}" "${BIN_DIR}"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

echo "Downloading ${URL}"
curl --fail --location --progress-bar "${URL}" -o "${tmp}/${ASSET}"

echo "Extracting"
tar -xzf "${tmp}/${ASSET}" -C "${INSTALL_DIR}"

ln -sf "${INSTALL_DIR}/bin/slangc" "${BIN_DIR}/slangc"

echo
echo "Done. slangc installed to ${INSTALL_DIR}/bin/slangc"
echo "Symlinked into ${BIN_DIR}/slangc"
echo

if ! command -v slangc >/dev/null 2>&1; then
  echo "NOTE: ${BIN_DIR} is not on your PATH. Add this to your shell rc:"
  echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
fi

"${INSTALL_DIR}/bin/slangc" -v || true
