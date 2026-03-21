#!/usr/bin/env bash
# terrascope/capture.sh — TerraScope microscope capture adapter
#
# Contract: device capture calls this with $1 = output directory.
# This script writes image + metadata files there.
# Exit 0 = success, non-zero = failure.

set -euo pipefail

OUTPUT_DIR="${1:?Usage: capture.sh <output-dir>}"

# --- Configuration (overridable via env) ---
RESOLUTION="${TERRASCOPE_RESOLUTION:-1920x1080}"
QUALITY="${TERRASCOPE_QUALITY:-90}"
IMAGE_NAME="capture.jpg"

# --- Auto-detect camera tool ---
detect_camera() {
  if command -v libcamera-still >/dev/null 2>&1; then
    echo "libcamera-still"
  elif command -v raspistill >/dev/null 2>&1; then
    echo "raspistill"
  else
    echo "ERROR: No camera tool found. Install libcamera-still or raspistill." >&2
    exit 1
  fi
}

# --- Capture image ---
capture_image() {
  local tool="$1" output_path="$2"
  local width height
  width="${RESOLUTION%x*}"
  height="${RESOLUTION#*x}"

  echo "Capturing with ${tool} (${RESOLUTION}, quality ${QUALITY})..."

  case "${tool}" in
    libcamera-still)
      libcamera-still \
        --width "${width}" --height "${height}" \
        --quality "${QUALITY}" \
        --nopreview \
        --output "${output_path}" \
        2>/dev/null
      ;;
    raspistill)
      raspistill \
        -w "${width}" -h "${height}" \
        -q "${QUALITY}" \
        -o "${output_path}"
      ;;
  esac
}

# --- Generate metadata ---
generate_metadata() {
  local tool="$1" image_path="$2" metadata_path="$3"
  local image_size timestamp hostname

  image_size=$(stat -c%s "${image_path}" 2>/dev/null || stat -f%z "${image_path}")
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  hostname=$(hostname)

  cat > "${metadata_path}" <<METADATA
{
  "camera_tool": "${tool}",
  "resolution": "${RESOLUTION}",
  "quality": ${QUALITY},
  "image_file": "${IMAGE_NAME}",
  "image_size_bytes": ${image_size},
  "captured_at": "${timestamp}",
  "hostname": "${hostname}",
  "terrascope_version": "1.0"
}
METADATA
}

# --- Main ---
main() {
  local camera_tool image_path metadata_path

  camera_tool="$(detect_camera)"
  image_path="${OUTPUT_DIR}/${IMAGE_NAME}"
  metadata_path="${OUTPUT_DIR}/metadata.json"

  capture_image "${camera_tool}" "${image_path}"

  [ -f "${image_path}" ] || {
    echo "ERROR: Image file not created at ${image_path}" >&2
    exit 1
  }

  generate_metadata "${camera_tool}" "${image_path}" "${metadata_path}"

  echo "Capture complete: ${image_path} + ${metadata_path}"
}

main "$@"
