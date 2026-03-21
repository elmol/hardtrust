# S2a.1-V2: Device — Capture Command

**Type:** Feature
**Story:** [S2a.1 Device Capture](../stories/slice-2/s2a.1-device-capture.md)
**Depends on:** S2a.1-V1 (protocol generic signing — `Capture` type + `sign<Capture>`)

---

## Problem

The device CLI only supports `emit` (temperature readings). TerraGenesis needs to capture microscopy images by executing an external command, hashing the output files, and producing a signed `capture.json`.

## What to Build

### Step 1 — Add `Capture` subcommand to device CLI

```rust
#[derive(Subcommand)]
enum Commands {
    Init,
    Emit,
    Capture {
        /// Command to execute for capturing data
        #[arg(long)]
        cmd: String,

        /// Directory where the capture command writes its output files
        #[arg(long, default_value = "./capture-output")]
        output_dir: PathBuf,
    },
}
```

### Step 2 — Implement capture flow

```
fn handle_capture(cmd: &str, output_dir: &Path) -> Result<(), Box<dyn Error>> {
    // 1. Create output_dir if it doesn't exist
    // 2. Execute cmd via std::process::Command (shell: bash -c "cmd")
    //    - If exit code != 0, print stderr and abort with clear error
    // 3. Read all files in output_dir (non-recursive, skip subdirs)
    //    - If empty, abort: "ERROR: No files found in {output_dir} after command"
    // 4. Sort files alphabetically by name
    // 5. For each file: compute SHA-256, collect CaptureFile { name, hash, size }
    // 6. Compute content_hash: SHA-256 of concatenated (name + individual_hash) for all files
    // 7. Read device key from ~/.hardtrust/device.key
    // 8. Build Capture struct (serial, address, timestamp, content_hash, files, signature="0x")
    // 9. Sign using protocol::sign(&key, &capture)
    // 10. Set capture.signature = signed value
    // 11. Write capture.json to current directory (not inside output_dir)
}
```

### Step 3 — Content hash algorithm

The `content_hash` is a deterministic hash of all output files:

```rust
fn compute_content_hash(files: &[CaptureFile]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    // files must already be sorted alphabetically by name
    for f in files {
        hasher.update(f.name.as_bytes());
        hasher.update(f.hash.strip_prefix("sha256:").unwrap().as_bytes());
    }
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
```

This ensures:
- Deterministic (sorted by name)
- Covers both file names and contents
- Cheap to compute (hashes of hashes, not re-reading files)

### Step 4 — Add `sha2` dependency to device/Cargo.toml

```toml
sha2 = "0.10"
```

Note: `sha2` (SHA-256) for file hashing, not `sha3` (keccak). SHA-256 is the standard for file integrity. keccak256 remains for the signing prehash (EVM compatibility).

---

## Output format

`capture.json`:

```json
{
  "serial": "TERRASCOPE-001",
  "address": "0x1234...",
  "timestamp": "2026-03-21T15:30:00Z",
  "content_hash": "sha256:abc123...",
  "files": [
    { "name": "image.jpg", "hash": "sha256:def456...", "size": 2048576 },
    { "name": "metadata.json", "hash": "sha256:789abc...", "size": 342 }
  ],
  "signature": "0x..."
}
```

---

## Files

| File | Action |
|------|--------|
| `device/src/main.rs` | Add `Capture` subcommand + `handle_capture()` |
| `device/Cargo.toml` | Add `sha2 = "0.10"` |

## Tests (TDD — write first)

1. **Capture with single file** — cmd writes one file, capture.json has correct hash and signature
2. **Capture with multiple files** — content_hash is deterministic regardless of filesystem order
3. **Capture command fails** — non-zero exit code aborts with error message
4. **Capture empty output** — no files after command → clear error
5. **Capture signature is valid** — round-trip: sign then verify using protocol::verify
6. **Capture preserves file manifest** — capture.json.files matches actual files in output dir
7. **Existing emit tests still pass** — no regressions

Use `tempfile::TempDir` for isolation. For the capture command in tests, use simple shell commands like `echo "test" > $OUTPUT_DIR/test.txt`.

## Validation Criteria

- [ ] `device capture --cmd "echo hello > output/test.txt" --output-dir ./output` produces valid `capture.json`
- [ ] `capture.json` signature verifies via `protocol::verify`
- [ ] Command failure (exit 1) produces clear error, no capture.json
- [ ] Empty output dir produces clear error
- [ ] `device emit` still works (no regression)
- [ ] `just ci` passes
