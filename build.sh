#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_NAME="mcps"
INSTALL_DIR="${MCPS_INSTALL_DIR:-$HOME/.local/bin}"

echo "Building ${BIN_NAME} (release)..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

mkdir -p "${INSTALL_DIR}"
install -m 0755 "${ROOT_DIR}/target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

echo "Installed ${BIN_NAME} to: ${INSTALL_DIR}/${BIN_NAME}"

case ":$PATH:" in
  *":${INSTALL_DIR}:"*)
    echo "PATH includes ${INSTALL_DIR}"
    ;;
  *)
    echo "PATH does not include ${INSTALL_DIR}"
    echo "Add this to your shell profile:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
