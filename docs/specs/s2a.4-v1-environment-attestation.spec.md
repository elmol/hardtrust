# S2a.4-V1: Capture Environment Attestation

**Type:** Feature
**Story:** [S2a.4 Capture Environment Attestation](../stories/slice-2/s2a.4-capture-environment-attestation.md)
**Depends on:** S2a.1-V2 (device capture), S2a.2-V1 (capture script)

---

## Problem

The capture pipeline delegates image acquisition to an external bash script (`capture.sh`). This script could be replaced or modified, allowing someone to inject fabricated images that still produce valid signatures. There is no way for the attester to verify that the official capture pipeline was used.

## What to Build

### Step 1 — Add `CaptureEnvironment` to protocol domain (`protocol/src/domain.rs`)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureEnvironment {
    pub script_hash: String,    // "sha256:<hex>"
    pub binary_hash: String,    // "sha256:<hex>"
    pub hw_serial: String,      // RPi serial (already available)
    pub camera_info: String,    // camera identifier string
}
```

Add `environment: CaptureEnvironment` field to existing `Capture` struct.

### Step 2 — Update `prehash()` for `Capture`

In the `Signable` impl for `Capture`, append environment fields to the hash input after `content_hash`:

```rust
impl Signable for Capture {
    fn prehash(&self) -> Option<[u8; 32]> {
        use sha3::{Keccak256, Digest};
        let mut hasher = Keccak256::new();
        hasher.update(self.serial.as_bytes());
        hasher.update(self.address.as_bytes());
        hasher.update(self.timestamp.as_bytes());
        hasher.update(self.content_hash.as_bytes());
        // environment attestation fields
        hasher.update(self.environment.script_hash.as_bytes());
        hasher.update(self.environment.binary_hash.as_bytes());
        hasher.update(self.environment.hw_serial.as_bytes());
        hasher.update(self.environment.camera_info.as_bytes());
        Some(hasher.finalize().into())
    }
    // ...
}
```

### Step 3 — Collect environment in device capture command

In the `capture` command handler, before signing, collect:

1. **`script_hash`**: Read the file at the `--cmd` path, compute SHA-256, format as `"sha256:<hex>"`.
   - If `--cmd` contains spaces/args (e.g. `"/path/to/script.sh arg1"`), hash only the script file (first token before space)
2. **`binary_hash`**: Read `/proc/self/exe` (the running binary), compute SHA-256, format as `"sha256:<hex>"`.
   - Fallback for non-Linux (tests): `"sha256:unknown"`
3. **`hw_serial`**: Already available from existing device serial logic.
4. **`camera_info`**: Best-effort detection:
   - Try `v4l2-ctl --list-devices` (first line of output)
   - Fallback: read `/proc/device-tree/model`
   - Fallback: `"unknown"`

### Step 4 — Extend attester `verify` with `--release-hashes`

Add optional flag to the existing `verify` command:

```rust
Verify {
    #[arg(long)]
    file: PathBuf,
    /// Path to SHA256SUMS file with known-good hashes for environment verification
    #[arg(long)]
    release_hashes: Option<PathBuf>,
}
```

SHA256SUMS file format (same as GitHub release artifact):
```
sha256:abc123...  device-armv7
sha256:def456...  terrascope-capture.sh
```

Verification logic after signature check:
1. Parse `environment` from the capture JSON
2. If `--release-hashes` provided:
   - Load the file, parse `hash  filename` lines
   - Match `environment.script_hash` against the entry containing `capture.sh`
   - Match `environment.binary_hash` against the entry containing `device`
   - Print `MATCH` or `MISMATCH` per field
3. If `--release-hashes` not provided:
   - Print the environment block as informational only

---

## Output format

`capture.json` with environment:

```json
{
  "serial": "TERRASCOPE-001",
  "address": "0x1234...",
  "timestamp": "2026-03-21T15:30:00Z",
  "content_hash": "sha256:abc123...",
  "files": [
    { "name": "image.jpg", "hash": "sha256:def456...", "size": 2048576 }
  ],
  "environment": {
    "script_hash": "sha256:aaa111...",
    "binary_hash": "sha256:bbb222...",
    "hw_serial": "10000000abcdef01",
    "camera_info": "mmal service 16.1"
  },
  "signature": "0x..."
}
```

Attester verify output:

```
Signature:   VERIFIED
Environment:
  script_hash:  sha256:aaa111... → MATCH (terrascope-capture.sh)
  binary_hash:  sha256:bbb222... → MATCH (device-armv7)
  hw_serial:    10000000abcdef01
  camera_info:  mmal service 16.1
```

---

## Files

| File | Action |
|------|--------|
| `protocol/src/domain.rs` | Add `CaptureEnvironment` struct, add `environment` field to `Capture` |
| `protocol/src/domain.rs` | Update `Signable` impl for `Capture` to include env fields in `prehash()` |
| `device/src/main.rs` (or `lib.rs`) | Collect environment before signing in capture handler |
| `attester/src/main.rs` (or `lib.rs`) | Add `--release-hashes` flag to verify, compare environment hashes |
| `scripts/mock-capture.sh` | May need update if environment collection affects mock flow |

## Tests (TDD — write first)

1. **CaptureEnvironment serde roundtrip** — serialize and deserialize CaptureEnvironment
2. **prehash includes environment** — changing any env field produces a different prehash
3. **Capture with environment** — capture.json includes `environment` block with all 4 fields
4. **script_hash correctness** — hash of --cmd file matches expected SHA-256
5. **binary_hash fallback** — on non-Linux (or test env), falls back to `"sha256:unknown"`
6. **Verify with matching release-hashes** — prints MATCH for script and binary
7. **Verify with mismatching release-hashes** — prints MISMATCH when hashes differ
8. **Verify without release-hashes** — prints environment as informational, no MATCH/MISMATCH
9. **E2E Case 5** — full capture + verify with `--release-hashes` → environment MATCH
10. **E2E Case 6** — capture with tampered script (copy + modify mock-capture.sh) + verify → environment MISMATCH, signature still VERIFIED
11. **Existing capture/emit tests still pass** — no regressions

## Validation Criteria

- [ ] `device capture --cmd "./scripts/mock-capture.sh"` produces `capture.json` with `environment` block
- [ ] `environment.script_hash` matches `sha256sum scripts/mock-capture.sh`
- [ ] `environment.binary_hash` is populated (or `sha256:unknown` in test env)
- [ ] Changing any environment field invalidates the signature (prehash coverage)
- [ ] `attester verify --file capture.json --release-hashes SHA256SUMS` shows MATCH/MISMATCH
- [ ] `attester verify --file capture.json` (no flag) shows environment as info only
- [ ] All existing emit and capture tests pass (no regression)
- [ ] `just ci` passes
