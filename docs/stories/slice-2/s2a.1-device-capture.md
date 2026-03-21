# S2a.1 — Device Capture

## User Story

As a TerraScope operator, I want my device to execute a capture command, hash all generated files, and produce a signed `capture.json`, so that I have cryptographic proof that specific files were produced by my device at a specific time.

## Acceptance Criteria

1. `device capture --cmd "./capture.sh" --output-dir ./output/` executes the command
2. After the command completes, device reads ALL files in the output directory
3. Device computes a `content_hash` (SHA-256 merkle of all files, sorted alphabetically by name)
4. Device signs `keccak256(serial_hash || address || content_hash || timestamp)`
5. Device writes `capture.json` with: serial, address, timestamp, content_hash, files manifest, signature
6. The capture command's exit code is respected — non-zero aborts with clear error
7. If output directory is empty after command, abort with clear error
8. Existing `emit` command continues working unchanged
9. No code duplication between `emit` and `capture` — shared signing lives in `protocol/`

## Out of Scope

- Attester verification of captures (Slice 2b)
- On-chain proof of capture (Slice 2c)
- Multiple captures in sequence
- Image format validation (device doesn't care what files the command produces)
