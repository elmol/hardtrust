# ADR-0009: Capture Script as Hardware Adapter

**Status:** Accepted
**Date:** 2026-03-21

## Context

TerraGenesis runs on TerraScope microscopes (Raspberry Pi). The `device` binary is hardware-agnostic — it hashes and signs files. The actual image capture is hardware-specific (camera APIs, sensor config, metadata).

We need a way to bridge the generic `device capture` command with TerraScope-specific camera hardware.

## Decision

The capture script is an **external executable** that `device capture --cmd` invokes. It acts as a hardware adapter between the generic device binary and specific microscope hardware.

### Contract between device and capture script

**Input:**
- Argument `$1`: output directory path (device creates it before calling the script)

**Output:**
- Script writes files to `$1` (images, metadata, whatever it produces)
- Exit code 0 = success, non-zero = failure (device aborts)
- stderr for error messages

**Device does NOT care about:**
- What files the script produces (images, JSON, CSV, anything)
- How many files
- What camera API is used
- What metadata format

### Repository location

```
terrascope/
  capture.sh    # TerraScope-specific hardware adapter
```

Separate from `device/` (generic) and `scripts/` (build/CI infrastructure). This makes the hardware-adapter boundary explicit.

### Installation path

The script installs to `/usr/local/lib/terrascope/capture.sh` via `install-device.sh`. The `device capture` command defaults to this path when `--cmd` is not specified.

### Default `--cmd`

```bash
# Common case (TerraScope on RPi) — just works:
device capture --output-dir ./output/

# Override for dev/testing/other hardware:
device capture --cmd "./my-script.sh" --output-dir ./output/
```

## Rationale

- **`--cmd` IS the plugin pattern** — no need for a registry, trait, or discovery mechanism
- **Versioned together** with the device binary — one release, one install, no compatibility matrix
- **`terrascope/` directory** separates concerns without premature abstraction
- **Default path** eliminates user configuration for the common case while keeping flexibility

## Consequences

- Adding a new hardware adapter = new directory + new script, no Rust changes needed
- The contract (input dir, exit code, files) must be documented and stable
- `install-device.sh` must be updated to copy the capture script
