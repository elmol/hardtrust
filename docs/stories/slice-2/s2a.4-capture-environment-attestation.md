# S2a.4 — Capture Environment Attestation

**Slice:** 2a
**Spec:** [S2a.4](../../specs/s2a.4-capture-environment.spec.md)

---

## User Story

As a **verifier**, I want each capture to include environment metadata (script hash, binary hash, hardware serial, camera info) so that I can detect if the capture software has been tampered with.

---

## Acceptance Criteria

1. `device capture` includes an `environment` object in `capture.json` with:
   - `script_hash` — SHA256 of the capture script
   - `binary_hash` — SHA256 of the device binary
   - `hw_serial` — Hardware serial number
   - `camera_info` — Camera model from device tree or v4l2-ctl
2. Environment fields are included in the signed prehash (tamper-evident)
3. `attester verify` compares environment hashes against:
   - On-chain approved hashes (if set via `set-release-hashes`)
   - Embedded release hashes (fallback)
4. Output shows MATCH or MISMATCH for each environment field

---

## Edge Cases

- No camera connected → `camera_info` = "unknown"
- No approved hashes set on-chain → fallback to embedded hashes
- Embedded hashes empty (debug build) → skip environment check with warning
- Tampered capture script → MISMATCH but signature still VERIFIED (environment is advisory)
