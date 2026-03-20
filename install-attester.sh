#!/usr/bin/env bash
# install-attester.sh — Install HardTrust `attester` on Ubuntu (x86_64) or macOS (arm64).
# For Raspberry Pi use install-device.sh instead.
#
# Env: HARDTRUST_VERSION (default: latest), INSTALL_DIR (default: /usr/local/bin)

set -euo pipefail
REPO="elmol/hardtrust"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY="attester"

TMP_DIR=""
cleanup() { [ -n "${TMP_DIR}" ] && rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

detect_target() {
  local os arch
  os="$(uname -s)"; arch="$(uname -m)"
  case "${os}" in
    Linux)
      case "${arch}" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        armv7l|armv7)
          echo "ERROR: This is a Raspberry Pi (ARMv7). Use install-device.sh." >&2; exit 1 ;;
        aarch64)
          echo "ERROR: Linux aarch64 not supported. See https://github.com/${REPO}/issues." >&2; exit 1 ;;
        *) echo "ERROR: Unsupported Linux arch: ${arch}." >&2; exit 1 ;;
      esac ;;
    Darwin)
      case "${arch}" in
        arm64) echo "aarch64-apple-darwin" ;;
        *) echo "ERROR: macOS ${arch} not supported. See https://github.com/${REPO}/issues." >&2; exit 1 ;;
      esac ;;
    *) echo "ERROR: Unsupported OS: ${os}." >&2; exit 1 ;;
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
    echo "WARNING: Skipping checksum." >&2; return 0
  fi
  [ "${actual}" = "${expected}" ] || {
    echo "ERROR: Checksum mismatch! Expected: ${expected} Got: ${actual}" >&2; exit 1; }
  echo "Checksum OK"
}

main() {
  echo "HardTrust Attester Installer (Linux x86_64 / macOS Apple Silicon)"
  echo "==================================================================="
  local target version artifact base_url
  target="$(detect_target)"
  version="$(resolve_version)"
  artifact="${BINARY}-${version}-${target}"
  base_url="https://github.com/${REPO}/releases/download/${version}"
  echo "Version: ${version} | Target: ${target}"
  TMP_DIR="$(mktemp -d)"
  echo "Downloading ${artifact} ..."
  curl -fsSL --progress-bar "${base_url}/${artifact}" -o "${TMP_DIR}/${BINARY}"
  local expected
  expected=$(curl -fsSL "${base_url}/${artifact}.sha256" | awk '{print $1}')
  verify_checksum "${TMP_DIR}/${BINARY}" "${expected}"
  echo "Installing to ${INSTALL_DIR}/${BINARY} ..."
  if [ -w "${INSTALL_DIR}" ]; then
    cp "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"; chmod +x "${INSTALL_DIR}/${BINARY}"
  else
    sudo cp "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"; sudo chmod +x "${INSTALL_DIR}/${BINARY}"
  fi
  echo "Done. Run 'attester --help' to get started."
}
main "$@"
