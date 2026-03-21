# S2a-Debt-V1: Slice 2a Review Fixes

**Type:** Bug fix / Tech debt
**Depends on:** S2a.1 + S2a.2 (all implemented)

---

## Problem

Post-review of Slice 2a found 6 issues (2 HIGH, 4 MEDIUM) that block a clean release.

---

## Fix 1 — Clean output directory before capture (HIGH)

`device capture` creates `--output-dir` but never cleans it. Running twice with the same dir includes stale files in the hash.

**File:** `device/src/main.rs` — in `handle_capture()`, before executing the command:

```rust
// Clean output dir to prevent stale files from previous captures
if output_dir.exists() {
    for entry in std::fs::read_dir(&output_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            std::fs::remove_file(entry.path())?;
        }
    }
}
```

**Test:** Run capture twice with same output-dir → second capture.json should NOT include files from first capture.

---

## Fix 2 — Replace string detection with serde untagged enum (HIGH)

`attester/src/main.rs` uses `json.contains("content_hash")` to detect file type. This is fragile.

**File:** `attester/src/main.rs` — replace the string detection with:

```rust
use hardtrust_protocol::{Capture, Reading, Signable};

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum DeviceData {
    Capture(Capture),
    Reading(Reading),
}
```

Then in the Verify handler:

```rust
Command::Verify { file, contract } => {
    let json = std::fs::read_to_string(&file)
        .map_err(|e| format!("could not read file {file}: {e}"))?;

    let data: DeviceData = serde_json::from_str(&json)
        .map_err(|e| format!("invalid JSON (expected reading or capture): {e}"))?;

    let (serial, verification) = match &data {
        DeviceData::Capture(c) => (&c.serial, c as &dyn Signable),
        DeviceData::Reading(r) => (&r.serial, r as &dyn Signable),
    };

    let reg = prepare_registration(serial);
    let provider = ProviderBuilder::new().connect_http(env_rpc_url()?);
    let registry = HardTrustRegistry::new(contract, &provider);
    let result = registry.getDevice(reg.serial_hash).call().await
        .map_err(|e| format!("contract query failed: {e}"))?;

    match verify_device_data(verification, result.deviceAddr) {
        VerificationResult::Verified => println!("VERIFIED"),
        VerificationResult::Unverified(_) => println!("UNVERIFIED"),
    }
}
```

**Note:** This requires `Signable` to be object-safe. If it isn't (due to generic methods), an alternative is to use a helper function that takes both types:

```rust
let (serial, result) = match data {
    DeviceData::Capture(c) => {
        let reg = prepare_registration(&c.serial);
        // ... query contract ...
        (c.serial.clone(), verify_device_data(&c, device.deviceAddr))
    }
    DeviceData::Reading(r) => {
        let reg = prepare_registration(&r.serial);
        // ... query contract ...
        (r.serial.clone(), verify_device_data(&r, device.deviceAddr))
    }
};
```

Either approach eliminates the string detection AND the duplicated RPC/contract logic.

---

## Fix 3 — Quote output_dir in shell command (MEDIUM)

`device/src/main.rs` passes output_dir unquoted to `bash -c`. Paths with spaces break.

**File:** `device/src/main.rs` — change the command construction:

```rust
// Before:
let full_cmd = format!("{} {}", cmd, output_dir.display());

// After:
let full_cmd = format!("{} '{}'", cmd, output_dir.display().to_string().replace('\'', "'\\''"));
```

The `replace` handles paths containing single quotes (edge case but correct).

**Test:** `device capture --cmd "echo test > $1/test.txt" --output-dir "./dir with spaces/"` should work.

---

## Fix 4 — Add checksum for capture script in install-device.sh (MEDIUM)

The device binary gets SHA-256 verification but the capture script does not.

**File:** `install-device.sh` — after downloading the capture script:

```bash
echo "Verifying capture script checksum..."
local capture_expected
capture_expected=$(curl -fsSL "${base_url}/terrascope-capture.sh.sha256" | awk '{print $1}')
verify_checksum "${CAPTURE_DIR}/capture.sh" "${capture_expected}"
```

**Also requires:** uploading a `terrascope-capture.sh.sha256` sidecar in release.yml — same pattern as the binary checksums.

**File:** `.github/workflows/release.yml` — in build-device-armv7 job, after the capture script upload, generate and upload the sha256:

```yaml
- name: Generate capture script checksum
  run: sha256sum terrascope/capture.sh | sed 's|terrascope/||' > terrascope-capture.sh.sha256

- name: Upload capture script checksum
  uses: actions/upload-artifact@v4
  with:
    name: terrascope-capture-sha256
    path: terrascope-capture.sh.sha256
```

---

## Fix 5 — Update README with capture documentation (MEDIUM)

**File:** `README.md` — add capture section after "The Wire" and update repo structure.

Add between "The Wire" and "Quick Start":

```markdown
## Capture — Microscopy Data Provenance

TerraGenesis extends the walking skeleton with microscopy image capture:

```bash
# On a TerraScope RPi with camera installed:
device capture --output-dir ./output/

# Verify the capture:
attester verify --file capture.json --contract <address>
```

What happens:
1. `device capture` calls the TerraScope capture script (`/usr/local/lib/terrascope/capture.sh`)
2. The script takes a photo and generates metadata
3. Device hashes all files, signs the content hash, writes `capture.json`
4. `attester verify` checks the signature against the on-chain registry
```

Update Repository Structure to include:

```
├── terrascope/         # TerraScope hardware adapter (capture script)
```

Check the "Hashing and data capture pipeline" item in Hackathon Scope.

---

## Fix 6 — Extract key-loading helper in device (MEDIUM)

**File:** `device/src/main.rs` — extract duplicated key-loading logic from Emit and Capture into a helper:

```rust
fn load_device_key() -> Result<(SigningKey, String, String), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
    let key_path = std::path::PathBuf::from(&home).join(".hardtrust/device.key");
    if !key_path.exists() {
        return Err("device key not found — run 'device init' first".into());
    }
    let key_hex = std::fs::read_to_string(&key_path)?;
    let key_bytes = hex::decode(key_hex.trim())?;
    let signing_key = SigningKey::from_slice(&key_bytes)?;
    let verifying_key = signing_key.verifying_key();
    let address = format!("{}", hardtrust_protocol::public_key_to_address(verifying_key));
    let serial = read_serial(); // existing function
    Ok((signing_key, serial, address))
}
```

Then both `handle_emit` and `handle_capture` call `load_device_key()`.

---

## Files Summary

| File | Fixes |
|------|-------|
| `device/src/main.rs` | Fix 1 (clean dir), Fix 3 (quote path), Fix 6 (extract key helper) |
| `attester/src/main.rs` | Fix 2 (serde untagged enum) |
| `install-device.sh` | Fix 4 (capture checksum) |
| `.github/workflows/release.yml` | Fix 4 (sha256 sidecar) |
| `README.md` | Fix 5 (capture docs) |

## Validation Criteria

- [ ] Capture with same output-dir twice → only new files in second capture.json
- [ ] `attester verify` works with both reading.json and capture.json (no string detection)
- [ ] Capture with spaces in output-dir path works
- [ ] `install-device.sh` verifies capture script checksum
- [ ] README documents capture workflow
- [ ] No duplicated key-loading code
- [ ] All existing tests pass
- [ ] `just ci` passes
