# S2a.2-V2: Default --cmd Path + Install Script Update

**Type:** Feature
**Story:** [S2a.2 Capture Script](../stories/slice-2/s2a.2-capture-script.md)
**Depends on:** S2a.2-V1 (capture script exists), S2a.1-V2 (device capture command)

---

## Problem

`device capture` requires `--cmd` explicitly every time. On a TerraScope RPi, the capture script is always the same. The install script doesn't include the capture script.

## What to Build

### Step 1 — Default `--cmd` in device CLI

In `device/src/main.rs`, make `--cmd` optional with a default:

```rust
Capture {
    /// Command to execute for capturing data.
    /// Defaults to /usr/local/lib/terrascope/capture.sh
    #[arg(long, default_value = "/usr/local/lib/terrascope/capture.sh")]
    cmd: String,

    /// Directory where the capture command writes its output files
    #[arg(long, default_value = "./capture-output")]
    output_dir: PathBuf,
},
```

Add a check before executing: if the default path doesn't exist, print a helpful error:

```
ERROR: Default capture script not found at /usr/local/lib/terrascope/capture.sh
       Install it with: curl -fsSL .../install-device.sh | bash
       Or specify a custom script: device capture --cmd ./my-script.sh
```

### Step 2 — Update `install-device.sh`

After installing the device binary, also install the capture script:

```bash
# Install capture script
CAPTURE_SCRIPT_URL="${base_url}/terrascope-capture.sh"
CAPTURE_DIR="/usr/local/lib/terrascope"

echo "Installing capture script to ${CAPTURE_DIR}/capture.sh ..."
if [ -w "$(dirname "${CAPTURE_DIR}")" ]; then
  mkdir -p "${CAPTURE_DIR}"
  curl -fsSL "${CAPTURE_SCRIPT_URL}" -o "${CAPTURE_DIR}/capture.sh"
  chmod +x "${CAPTURE_DIR}/capture.sh"
else
  sudo mkdir -p "${CAPTURE_DIR}"
  sudo curl -fsSL "${CAPTURE_SCRIPT_URL}" -o "${CAPTURE_DIR}/capture.sh"
  sudo chmod +x "${CAPTURE_DIR}/capture.sh"
fi
```

### Step 3 — Add capture script to release pipeline

In `.github/workflows/release.yml`, upload `terrascope/capture.sh` as a release artifact named `terrascope-capture.sh` in the `build-device-armv7` job:

```yaml
- name: Upload capture script
  uses: actions/upload-artifact@v4
  with:
    name: terrascope-capture
    path: terrascope/capture.sh
```

And in the `release` job, include it in the GitHub Release assets.

---

## Files

| File | Action |
|------|--------|
| `device/src/main.rs` | Make `--cmd` optional with default path, add missing-script error |
| `install-device.sh` | Add capture script download + install |
| `.github/workflows/release.yml` | Upload capture script as release artifact |

## Tests (TDD)

1. `device capture` without `--cmd` uses default path
2. `device capture` with missing default script → clear error message
3. `device capture --cmd ./custom.sh` still works (override)
4. Existing capture tests pass unchanged

## Validation Criteria

- [ ] `device capture --output-dir ./out/` on RPi with installed script → works
- [ ] `device capture --output-dir ./out/` without installed script → helpful error
- [ ] `device capture --cmd ./test.sh --output-dir ./out/` → override works
- [ ] `install-device.sh` installs both binary + capture script
- [ ] Release pipeline includes `terrascope-capture.sh` in assets
- [ ] `just ci` passes
