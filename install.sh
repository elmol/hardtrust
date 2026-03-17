#!/usr/bin/env bash
# install.sh — HardTrust binary installer
#
# Detects the current platform, downloads the correct binary from GitHub Releases,
# verifies its SHA256 checksum, and installs it to INSTALL_DIR.
#
# Environment variables:
#   HARDTRUST_VERSION  — release tag to install (default: latest)
#   HARDTRUST_BINARY   — "attester" or "device" (default: auto-detect from platform)
#   INSTALL_DIR        — installation directory (default: /usr/local/bin)

set -euo pipefail

REPO="elmol/hardtrust"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ---------------------------------------------------------------------------
# Platform detection → Rust target triple
# ---------------------------------------------------------------------------
detect_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}" in
    Linux)
      case "${arch}" in
        x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        armv7l)  echo "armv7-unknown-linux-musleabihf" ;;
        armv7)   echo "armv7-unknown-linux-musleabihf" ;;
        *)
          echo "ERROR: Unsupported Linux architecture: ${arch}" >&2
          echo "       Supported: x86_64, armv7l (Raspberry Pi ARMv7)" >&2
          exit 1 ;;
      esac ;;
    Darwin)
      case "${arch}" in
        arm64)   echo "aarch64-apple-darwin" ;;
        *)
          echo "ERROR: Unsupported macOS architecture: ${arch}" >&2
          echo "       Supported: arm64 (Apple Silicon)" >&2
          echo "       Open an issue: https://github.com/${REPO}/issues" >&2
          exit 1 ;;
      esac ;;
    *)
      echo "ERROR: Unsupported OS: ${os}" >&2
      exit 1 ;;
  esac
}

# ---------------------------------------------------------------------------
# Binary auto-selection based on platform
# ---------------------------------------------------------------------------
select_binary() {
  local platform="$1"

  if [ -n "${HARDTRUST_BINARY:-}" ]; then
    echo "${HARDTRUST_BINARY}"
    return
  fi

  case "${platform}" in
    armv7-unknown-linux-musleabihf)  echo "device" ;;
    x86_64-unknown-linux-gnu)        echo "attester" ;;
    aarch64-apple-darwin)            echo "attester" ;;
    *)
      echo "ERROR: Cannot auto-select binary for platform: ${platform}" >&2
      echo "       Set HARDTRUST_BINARY=attester or HARDTRUST_BINARY=device" >&2
      exit 1 ;;
  esac
}

# ---------------------------------------------------------------------------
# Resolve "latest" via GitHub API
# ---------------------------------------------------------------------------
resolve_version() {
  if [ -n "${HARDTRUST_VERSION:-}" ]; then
    echo "${HARDTRUST_VERSION}"
    return
  fi

  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

  if [ -z "${latest}" ]; then
    echo "ERROR: Could not resolve latest release." >&2
    echo "       Set HARDTRUST_VERSION=v0.1.0 or check https://github.com/${REPO}/releases" >&2
    exit 1
  fi

  echo "${latest}"
}

# ---------------------------------------------------------------------------
# SHA256 checksum verification
# ---------------------------------------------------------------------------
verify_checksum() {
  local file="$1"
  local expected="$2"
  local actual

  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "${file}" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "${file}" | awk '{print $1}')
  else
    echo "WARNING: sha256sum and shasum not found — skipping integrity check." >&2
    return 0
  fi

  if [ "${actual}" != "${expected}" ]; then
    echo "ERROR: Checksum mismatch!" >&2
    echo "  Expected: ${expected}" >&2
    echo "  Got:      ${actual}" >&2
    echo "  The download may be corrupted. Delete the file and retry." >&2
    exit 1
  fi

  echo "Checksum OK"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
  echo "HardTrust Installer"
  echo "==================="

  local platform binary version artifact base_url
  platform="$(detect_platform)"
  binary="$(select_binary "${platform}")"
  version="$(resolve_version)"
  artifact="${binary}-${version}-${platform}"
  base_url="https://github.com/${REPO}/releases/download/${version}"

  echo "Platform:  ${platform}"
  echo "Binary:    ${binary}"
  echo "Version:   ${version}"
  echo ""

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "${tmp_dir}"' EXIT

  echo "Downloading ${artifact} ..."
  curl -fsSL --progress-bar "${base_url}/${artifact}" -o "${tmp_dir}/${binary}"

  echo "Verifying checksum ..."
  local checksum_line expected
  checksum_line=$(curl -fsSL "${base_url}/${artifact}.sha256")
  expected=$(echo "${checksum_line}" | awk '{print $1}')
  verify_checksum "${tmp_dir}/${binary}" "${expected}"

  echo ""
  echo "Installing to ${INSTALL_DIR}/${binary} ..."
  if [ -w "${INSTALL_DIR}" ]; then
    cp "${tmp_dir}/${binary}" "${INSTALL_DIR}/${binary}"
    chmod +x "${INSTALL_DIR}/${binary}"
  else
    echo "(requires sudo for ${INSTALL_DIR})"
    sudo cp "${tmp_dir}/${binary}" "${INSTALL_DIR}/${binary}"
    sudo chmod +x "${INSTALL_DIR}/${binary}"
  fi

  echo ""
  echo "Installed: $(command -v "${binary}")"
  "${binary}" --help 2>/dev/null | head -3 || true
  echo ""
  echo "Done. Run '${binary} --help' to get started."
}

main "$@"
