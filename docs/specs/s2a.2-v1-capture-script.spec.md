# S2a.2-V1: TerraScope Capture Script

**Type:** Feature
**Story:** [S2a.2 Capture Script](../stories/slice-2/s2a.2-capture-script.md)
**Depends on:** S2a.1-V2 (device capture command exists)

---

## Problem

The current capture script is a single line: `raspistill -o "$file"`. It has no error handling, no metadata, and `raspistill` is deprecated on newer Raspberry Pi OS (replaced by `libcamera-still`).

## What to Build

### `terrascope/capture.sh`

```bash
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
```

### Key improvements over the original:
- **Auto-detect** `libcamera-still` vs `raspistill` (future-proof)
- **Configurable** resolution and quality via env vars
- **Metadata** JSON with camera info, resolution, timestamp, hostname
- **Error handling** — fails cleanly if no camera, if capture fails, if image not created
- **Follows the contract** from ADR-0009: takes `$1` as output dir, writes files there

---

## Files

| File | Action |
|------|--------|
| `terrascope/capture.sh` | Create — TerraScope hardware adapter |
| `docs/adr/0009-capture-script-hardware-adapter.md` | Create — documents the adapter pattern |

## Validation Criteria

- [ ] `terrascope/capture.sh ./test-output/` captures image + metadata on RPi with camera
- [ ] Auto-detects `libcamera-still` or `raspistill`
- [ ] Exits non-zero with clear error if no camera tool
- [ ] `metadata.json` has all required fields
- [ ] `TERRASCOPE_RESOLUTION=640x480 terrascope/capture.sh ./out/` respects override
- [ ] `device capture --cmd "./terrascope/capture.sh" --output-dir ./out/` produces valid `capture.json`
