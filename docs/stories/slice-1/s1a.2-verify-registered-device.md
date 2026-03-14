# S1a.2 — Verify Data from a Registered Device

## User Story

As anyone, I can verify that a data reading comes from a registered device, so that I can confirm the data is from a trusted source.

## Context

Building on S1a.1 (device registered on-chain), this slice adds data emission and verification. The device emits a hardcoded reading to a JSON file. The verifier reads the file, checks the device address against the on-chain registry, and gets VERIFIED. No signature verification — address match only.

## Acceptance Criteria

- [ ] Running `device emit` writes a `reading.json` with serial, address, temperature (hardcoded), timestamp (real UTC), and signature (fake placeholder)
- [ ] Running `attester verify --file reading.json` reads the JSON, queries the contract, and prints **VERIFIED**
- [ ] The full chain runs in sequence: init → deploy → register → emit → verify = **VERIFIED**

## Out of Scope

- UNVERIFIED case → S1a.3
- Real signature verification (ECDSA) → S1b
- Tampered data detection (INVALID) → S1b
- Error handling for missing files, malformed JSON → S1c

## Related ADRs

- ADR-0003: Alloy for Rust-EVM bindings
- ADR-0005: Hybrid storage (readings off-chain as JSON)

## Definition of Done

- A registered device emits data and verification returns VERIFIED
- Tests cover the VERIFIED path
- The sequence is reproducible from a clean Anvil
