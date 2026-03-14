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

**Goal:** Replace hardcoded values with real behavior. Introduce secp256k1 crypto.

Stories: TBD (will be defined when S1a is complete)

---

## S1c — "Polish & CI"

**Goal:** Edge cases, CI integration, e2e automation.

Stories: TBD (will be defined when S1b is complete)

**Exit gate:** Slice 1 gate from story map is met.
