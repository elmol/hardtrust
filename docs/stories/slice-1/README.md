# Slice 1 — CLI/Console (End-to-End Proof)

Each sub-slice is implemented as **horizontal stories** — thin end-to-end passes across all layers (device, contracts, attester). Each story delivers demonstrable value.

## S1a — "The Wire"

**Goal:** Prove bytes flow end-to-end. Everything hardcoded, no crypto.

| Story | Name | Delivers |
|-------|------|----------|
| [S1a.1](s1a.1-register-device-on-chain.md) | Register device and confirm on-chain | Device identity exists in the registry |
| [S1a.2](s1a.2-verify-registered-device.md) | Verify data from registered device | VERIFIED — trust confirmed |
| [S1a.3](s1a.3-verify-unregistered-device.md) | Verify data from unregistered device | UNVERIFIED — contrast proven |

**Exit gate:** VERIFIED for registered, UNVERIFIED for unregistered. Full chain in a terminal.

---

## S1b — "Real Crypto"

**Goal:** Replace hardcoded values with real secp256k1 cryptography. Each story is a horizontal slice across all affected layers.

| Story | Name | Delivers |
|-------|------|----------|
| S1b.1 | Real Keys | `device init` generates real keypair, persists `~/.hardtrust/device.key`, prints real serial + derived address |
| S1b.2 | Signed Reading | `device emit` produces a real ECDSA signature over the canonical payload; `reading.json` has a real hex sig |
| S1b.3 | Real Verify | `attester verify` recovers signer address via ecrecover, compares to on-chain address — not just equality check |

**Technical decisions:**
- Crypto library: `k256` crate for key gen + signing; Alloy `Signature::recover_address_from_prehash` for verification
- Key storage: raw 32-byte hex string, `~/.hardtrust/device.key`, permissions `0600`
- Signing payload: keccak256 of canonical tuple `(serial_hash, address, temperature_scaled_i64, timestamp_u64)` — no Ethereum personal sign prefix
- `common/` crate becomes real: houses `public_key_to_address`, `sign_reading`, `verify_reading`
- On-chain ecrecover deferred to S1c

**Exit gate:** Full `just e2e` passes. Corrupting any reading field causes `UNVERIFIED`. `cargo test --workspace` passes including signature round-trip tests.

---

## S1c — "Polish & CI"

**Goal:** Edge cases, CI integration, e2e automation.

Stories: TBD (will be defined when S1b is complete)

**Exit gate:** Slice 1 gate from story map is met.
