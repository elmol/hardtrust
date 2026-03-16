# ADR-0007 — No Ethereum Personal Sign Prefix for Device Readings

## Status

Accepted

## Context

When signing arbitrary bytes with an Ethereum private key, the convention established
by EIP-191 / `eth_sign` is to prepend `\x19Ethereum Signed Message:\n32` to the
32-byte hash before signing. This prefix exists to prevent a user from unknowingly
signing a valid Ethereum transaction when asked to sign a message in a wallet UI.

Device readings in HardTrust are **machine-to-machine payloads**. They are never
presented to a human in a wallet context and can never be confused with an Ethereum
transaction. The signer is an embedded device acting autonomously; there is no
wallet, no user, and no transaction risk.

The canonical payload is a tightly specified 68-byte preimage:

```
keccak256(serial) || address_bytes(20) || temperature*1000 as i64 BE(8) || timestamp as u64 BE(8)
```

The final hash signed is `keccak256(preimage)` — raw prehash, no prefix.

## Decision

`sign_reading` and `verify_reading` in `hardtrust-types` use **raw prehash signing**
with no EIP-191 personal sign prefix.

## Consequences

- Simpler implementation: no prefix concatenation on device or verifier.
- The `sign_reading` / `verify_reading` signatures are **not** compatible with
  `eth_sign` or EIP-712 typed data — they are a custom format specific to HardTrust.
- Any future off-chain tool or contract verifier must reconstruct the same canonical
  hash (no prefix) to verify a reading signature correctly.
- If a smart-contract `ecrecover` path is added in future stories, it must call
  `ecrecover(keccak256(preimage), v, r, s)` directly — not use OpenZeppelin's
  `MessageHashUtils.toEthSignedMessageHash`.
