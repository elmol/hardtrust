#!/usr/bin/env bash
# build-attester.sh — Build `attester` for the current host platform.
#
# Supported hosts: x86_64-unknown-linux-gnu, aarch64-apple-darwin
# Required env: RELEASE_VERSION (e.g. "v0.1.0" or "dev")
# Output: attester-${RELEASE_VERSION}-${TARGET} + .sha256

set -euo pipefail

if [ -z "${RELEASE_VERSION:-}" ]; then
  echo "ERROR: RELEASE_VERSION is not set." >&2
  echo "       Usage: RELEASE_VERSION=v0.1.0 bash scripts/build-attester.sh" >&2
  exit 1
fi

detect_target() {
  local os arch
  os="$(uname -s)"; arch="$(uname -m)"
  case "${os}" in
    Linux)
      case "${arch}" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        *) echo "ERROR: Unsupported Linux arch: ${arch}. Supported: x86_64." >&2; exit 1 ;;
      esac ;;
    Darwin)
      case "${arch}" in
        arm64) echo "aarch64-apple-darwin" ;;
        *) echo "ERROR: Unsupported macOS arch: ${arch}. Supported: arm64." >&2; exit 1 ;;
      esac ;;
    *) echo "ERROR: Unsupported OS: ${os}." >&2; exit 1 ;;
  esac
}

TARGET="$(detect_target)"

if ! rustup target list --installed | grep -q "^${TARGET}$"; then
  echo "ERROR: Rust target '${TARGET}' not installed." >&2
  echo "       Run: rustup target add ${TARGET}" >&2
  exit 1
fi

echo "==> Building contracts (ABI required by protocol crate) ..."
cd contracts && forge build && cd ..

echo "==> Building attester for ${TARGET} ..."
cargo build --release --package attester --target "${TARGET}"

ARTIFACT="attester-${RELEASE_VERSION}-${TARGET}"
echo "==> Creating artifact: ${ARTIFACT}"
cp "target/${TARGET}/release/attester" "${ARTIFACT}"

echo "==> Generating checksum ..."
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "${ARTIFACT}" > "${ARTIFACT}.sha256"
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "${ARTIFACT}" > "${ARTIFACT}.sha256"
else
  echo "ERROR: sha256sum/shasum not found." >&2; exit 1
fi

echo ""
echo "Build complete."
echo "  Artifact: ${ARTIFACT}"
echo "  Target:   ${TARGET}"
echo "  Size:     $(du -sh "${ARTIFACT}" | cut -f1)"
