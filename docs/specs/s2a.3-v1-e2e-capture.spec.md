# S2a.3-V1: E2E — Capture Verification Cases

**Type:** Test
**Story:** [S2a.3 E2E Capture](../stories/slice-2/s2a.3-e2e-capture.md)
**Depends on:** S2a.1-V2 (device capture), S2a.1-V3 (attester verify capture)

---

## Problem

The e2e test (`scripts/e2e-the-wire.sh`) only covers the reading flow (emit → verify). The capture flow (capture → verify) has no end-to-end coverage. Since `emit` will eventually be replaced by `capture`, the e2e must validate the capture pipeline.

## What to Build

### Step 1 — Create `scripts/mock-capture.sh`

A deterministic mock that emulates a TerraScope capture without camera hardware. Follows the ADR-0009 contract: takes `$1` as output dir, writes files there.

```bash
#!/usr/bin/env bash
# mock-capture.sh — Emulates a TerraScope capture for testing.
# Produces a fake image and metadata in the output directory.
# Follows ADR-0009 contract: $1 = output directory.

set -euo pipefail

OUTPUT_DIR="${1:?Usage: mock-capture.sh <output-dir>}"

# Generate a deterministic fake image (random-ish but reproducible from hostname)
echo "TERRASCOPE-MOCK-IMAGE-$(hostname)-$(date +%s)" > "${OUTPUT_DIR}/capture.jpg"

# Generate metadata
cat > "${OUTPUT_DIR}/metadata.json" <<EOF
{
  "camera_tool": "mock",
  "resolution": "640x480",
  "quality": 90,
  "image_file": "capture.jpg",
  "captured_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "hostname": "$(hostname)",
  "terrascope_version": "mock-1.0"
}
EOF

echo "Mock capture complete: ${OUTPUT_DIR}/capture.jpg + ${OUTPUT_DIR}/metadata.json"
```

Make executable: `git update-index --chmod=+x scripts/mock-capture.sh`

### Step 2 — Extend `scripts/e2e-the-wire.sh`

Add two cases after the existing Case 2, before cleanup:

```bash
# === CASE 3: VERIFIED capture (registered device) ===
echo ""
echo "=== Case 3: Capture VERIFIED ==="

# Clean any previous capture output
rm -rf capture-output capture.json

cargo run --bin device -- capture \
  --cmd "./scripts/mock-capture.sh" \
  --output-dir ./capture-output

echo "Capture written"

VERIFY_CAPTURE=$(cargo run --bin attester -- verify \
  --file capture.json \
  --contract "$CONTRACT_ADDRESS")
echo "$VERIFY_CAPTURE"

if [[ "$VERIFY_CAPTURE" != *"VERIFIED"* ]]; then
    echo "The Wire gate: FAILED — expected VERIFIED for capture from registered device"
    exit 1
fi
echo "Case 3: Capture VERIFIED — OK"

# === CASE 4: UNVERIFIED capture (fake capture) ===
echo ""
echo "=== Case 4: Capture UNVERIFIED ==="

cat > fake-capture.json <<'FAKEJSON'
{
  "serial": "FAKE-SCOPE-999",
  "address": "0x0000000000000000000000000000000000000BAD",
  "timestamp": "2025-01-01T00:00:00Z",
  "content_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "files": [
    { "name": "fake.jpg", "hash": "sha256:1111111111111111111111111111111111111111111111111111111111111111", "size": 100 }
  ],
  "signature": "0xFAKESIG"
}
FAKEJSON

UNVERIFIED_CAPTURE=$(cargo run --bin attester -- verify \
  --file fake-capture.json \
  --contract "$CONTRACT_ADDRESS")
echo "$UNVERIFIED_CAPTURE"

if [[ "$UNVERIFIED_CAPTURE" == *"VERIFIED"* && "$UNVERIFIED_CAPTURE" != *"UNVERIFIED"* ]]; then
    echo "The Wire gate: FAILED — expected UNVERIFIED for fake capture"
    exit 1
fi
echo "Case 4: Capture UNVERIFIED — OK"
```

### Step 3 — Update cleanup section

Add capture artifacts to the cleanup at the end:

```bash
# Cleanup
rm -f fake-reading.json fake-capture.json capture.json reading.json
rm -rf capture-output
```

### Step 4 — Update success message

```bash
echo ""
echo "The Wire gate: PASSED (4 cases — reading verified/unverified + capture verified/unverified)"
```

---

## Files

| File | Action |
|------|--------|
| `scripts/mock-capture.sh` | Create — deterministic mock capture script |
| `scripts/e2e-the-wire.sh` | Extend — add Cases 3 + 4 for capture |

## Validation Criteria

- [ ] `just e2e-the-wire` passes with all 4 cases
- [ ] Case 3: device capture with mock → attester verify → VERIFIED
- [ ] Case 4: fake capture.json → attester verify → UNVERIFIED
- [ ] Cases 1 + 2 (reading) still pass unchanged
- [ ] CI passes (no camera hardware needed)
- [ ] Mock script follows ADR-0009 contract
