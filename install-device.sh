#!/usr/bin/env bash
# install-device.sh — Install HardTrust `device` on Raspberry Pi (ARMv7 only).
# For Ubuntu/macOS use install-attester.sh instead.
#
# Env: HARDTRUST_VERSION (default: latest), INSTALL_DIR (default: /usr/local/bin)

set -euo pipefail
REPO="elmol/hardtrust"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY="device"
TARGET="armv7-unknown-linux-musleabihf"

guard_platform() {
  local os arch
  os="$(uname -s)"; arch="$(uname -m)"
  [ "${os}" = "Linux" ] || {
    echo "ERROR: This script is only for Raspberry Pi (Linux ARMv7)." >&2
    echo "       For Ubuntu/macOS use install-attester.sh." >&2; exit 1; }
  case "${arch}" in
    armv7l|armv7) ;;  # OK
    x86_64)
      echo "ERROR: This is a server/desktop (x86_64). Use install-attester.sh." >&2; exit 1 ;;
    aarch64)
      echo "ERROR: ARMv7 32-bit required, detected aarch64. Check https://github.com/${REPO}/issues." >&2; exit 1 ;;
    *)
      echo "ERROR: Unsupported arch: ${arch}." >&2; exit 1 ;;
  esac
}

resolve_version() {
  [ -n "${HARDTRUST_VERSION:-}" ] && { echo "${HARDTRUST_VERSION}"; return; }

  local v
  # Use /releases (not /releases/latest) so pre-releases are included
  v=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" \
    | grep '"tag_name"' \
    | head -1 \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

  [ -n "${v}" ] || {
    echo "ERROR: Could not resolve latest release." >&2
    echo "       Set HARDTRUST_VERSION=v0.1.0-rc8 or check https://github.com/${REPO}/releases" >&2
    exit 1
  }
  echo "${v}"
}

verify_checksum() {
  local file="$1" expected="$2" actual
  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "${file}" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "${file}" | awk '{print $1}')
  else
    echo "WARNING: Skipping checksum (sha256sum/shasum not found)." >&2; return 0
  fi
  [ "${actual}" = "${expected}" ] || {
    echo "ERROR: Checksum mismatch! Expected: ${expected} Got: ${actual}" >&2; exit 1; }
  echo "Checksum OK"
}

main() {
  echo "HardTrust Device Installer (Raspberry Pi ARMv7)"
  echo "================================================"
  guard_platform
  local version artifact base_url
  version="$(resolve_version)"
  artifact="${BINARY}-${version}-${TARGET}"
  base_url="https://github.com/${REPO}/releases/download/${version}"
  echo "Version: ${version} | Target: ${TARGET}"
  local tmp; tmp="$(mktemp -d)"; trap 'rm -rf "${tmp}"' EXIT
  echo "Downloading ${artifact} ..."
  curl -fsSL --progress-bar "${base_url}/${artifact}" -o "${tmp}/${BINARY}"
  local expected
  expected=$(curl -fsSL "${base_url}/${artifact}.sha256" | awk '{print $1}')
  verify_checksum "${tmp}/${BINARY}" "${expected}"
  echo "Installing to ${INSTALL_DIR}/${BINARY} ..."
  if [ -w "${INSTALL_DIR}" ]; then
    cp "${tmp}/${BINARY}" "${INSTALL_DIR}/${BINARY}"; chmod +x "${INSTALL_DIR}/${BINARY}"
  else
    sudo cp "${tmp}/${BINARY}" "${INSTALL_DIR}/${BINARY}"; sudo chmod +x "${INSTALL_DIR}/${BINARY}"
  fi
  echo "Done. Run 'device --help' to get started."
}
main "$@"
