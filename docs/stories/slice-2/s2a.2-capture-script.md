# S2a.2 — TerraScope Capture Script

## User Story

As a TerraScope operator, I want `device capture` to automatically take a photo with the microscope camera and collect metadata, so that I can produce signed captures without manually running camera commands.

## Acceptance Criteria

1. `device capture --output-dir ./output/` on a RPi with camera takes a photo and produces a signed `capture.json`
2. The capture script produces at least two files: the image and a metadata JSON
3. Metadata includes: timestamp, camera tool used, resolution, image filename, hostname
4. The script works with both `libcamera-still` (modern) and `raspistill` (legacy), auto-detecting which is available
5. If no camera is available, the script exits non-zero with a clear error
6. The script is installed to `/usr/local/lib/terrascope/capture.sh` via `install-device.sh`
7. `device capture` defaults to `/usr/local/lib/terrascope/capture.sh` when `--cmd` is not specified
8. `--cmd` override still works for dev/testing

## Out of Scope

- Multiple image capture in one invocation
- Video capture
- Image processing or compression
- GPS/location metadata (future)
