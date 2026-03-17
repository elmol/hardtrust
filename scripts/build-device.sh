#!/usr/bin/env bash
# build-device.sh — Cross-compile `device` for ARMv7 musl (Raspberry Pi).
#
# Prerequisites: Rust target armv7-unknown-linux-musleabihf, cargo-zigbuild, zig
# Required env: RELEASE_VERSION (e.g. "v0.1.0" or "dev")
# Output: device-${RELEASE_VERSION}-armv7-unknown-linux-musleabihf + .sha256

set -euo pipefail

TARGET="armv7-unknown-linux-musleabihf"

if [ -z "${RELEASE_VERSION:-}" ]; then
  echo "ERROR: RELEASE_VERSION is not set." >&2
  echo "       Usage: RELEASE_VERSION=v0.1.0 bash scripts/build-device.sh" >&2
  exit 1
fi

if ! command -v zig >/dev/null 2>&1; then
  echo "ERROR: 'zig' not found. Install Zig: https://ziglang.org/download/" >&2
  exit 1
fi

if ! command -v cargo-zigbuild >/dev/null 2>&1; then
  echo "ERROR: 'cargo-zigbuild' not found in PATH." >&2
  echo "       PATH=${PATH}" >&2
  echo "       CARGO_HOME=${CARGO_HOME:-unset}" >&2
  echo "       Install: cargo install cargo-zigbuild" >&2
  exit 1
fi

if ! rustup target list --installed | grep -q "^${TARGET}$"; then
  echo "ERROR: Rust target '${TARGET}' not installed." >&2
  echo "       Run: rustup target add ${TARGET}" >&2
  exit 1
fi

echo "==> Building contracts (ABI required by protocol crate) ..."
cd contracts && forge build && cd ..

echo "==> Cross-compiling device for ${TARGET} ..."
cargo zigbuild --release --package device --target "${TARGET}"

ARTIFACT="device-${RELEASE_VERSION}-${TARGET}"
echo "==> Creating artifact: ${ARTIFACT}"
cp "target/${TARGET}/release/device" "${ARTIFACT}"

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
echo "  Size:     $(du -sh "${ARTIFACT}" | cut -f1)"
