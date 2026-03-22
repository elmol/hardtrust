# ADR-0010: On-Chain Capture Verification Model

## Status

Proposed

## Context

TerraGenesis captures are signed by device keys and currently verified off-chain by the attester CLI using Rust ecrecover. The verification result is ephemeral -- you must trust whoever ran the attester. For trustless verification, the check needs to move on-chain.

There are several ways to implement on-chain verification, each with different trade-offs for gas cost, complexity, and what guarantees they provide. The key design question is: **does the contract just verify signatures (stateless), or does it also persist proof records (stateful)?**

A secondary question is whether the contract should reconstruct the capture hash from raw fields (serial, address, content_hash, timestamp, environment) or receive a pre-computed hash. Reconstructing on-chain provides stronger guarantees but costs significantly more gas and requires the contract to know the hash encoding scheme.

## Decision

**Split on-chain verification into two specs with incremental delivery:**

### S2b.1 -- View-only verification (no state change)

Add a `verifyCapture(bytes32 captureHash, uint8 v, bytes32 r, bytes32 s)` view function that:
1. Calls `ecrecover` to recover the signer
2. Checks signer against a `registeredDevices` reverse lookup mapping
3. Returns `(bool valid, address signer)` -- no state change, no gas cost for callers

The contract receives a pre-computed `captureHash` (the keccak256 prehash from the Rust `Signable` trait). It does NOT reconstruct the hash from raw fields. Rationale: hash reconstruction on-chain would require encoding environment fields of variable length, making the contract tightly coupled to the Rust domain model. The signature already binds the hash to the signer -- if the hash is wrong, verification fails.

### S2b.2 (future) -- Proof persistence

A separate `submitProof` function that writes a capture record on-chain (hash, signer, timestamp, block number). This is deferred because it requires gas expenditure per capture and the economic model is not yet defined.

### Signature security

Use OpenZeppelin's `ECDSA.recover()` instead of raw `ecrecover` to get built-in protections:
- s-value malleability rejection (enforces low-s)
- v-value validation (27 or 28 only)
- Zero address check

This is important even for a view function because S2b.2 will build on this function, and malleability in a stateful context could allow duplicate proof submissions.

### Prehash compatibility

The Rust side uses `sign_prehash_recoverable` (raw keccak256 hash, no EIP-191 prefix). The Solidity side uses `ecrecover` (also raw hash, no prefix). These are compatible. The spec MUST include a shared test vector to verify both sides produce the same hash for identical inputs.

## Alternatives Considered

### A. Full on-chain hash reconstruction

The contract receives raw capture fields (serial, content_hash, timestamp, environment fields) and reconstructs the keccak256 hash on-chain before running ecrecover.

- **Pro:** Stronger guarantee -- the contract independently verifies what was signed
- **Con:** High gas cost for variable-length environment fields (~50k+ gas), tight coupling between contract and Rust domain model, any prehash change requires contract redeployment
- **Verdict:** Rejected. The signature already cryptographically binds the hash to the content. If you trust the hash was computed correctly (which the Rust code does), passing the pre-computed hash is sufficient.

### B. EIP-712 typed structured data

Use EIP-712 `eth_signTypedData` with a domain separator and typed struct hash. The contract defines the struct type and reconstructs the hash using `hashStruct`.

- **Pro:** Standard approach, wallet-friendly, domain separation prevents cross-contract replay
- **Con:** Requires changing the Rust signing side to use EIP-712 encoding instead of raw keccak256. The device uses k256 (not a wallet), so EIP-712 wallet UX benefits don't apply. Adds complexity for no practical gain in a device-signing context.
- **Verdict:** Deferred. Could be valuable if captures need to be verified by third-party contracts or wallets in the future. Not justified for the current device-to-attester flow.

### C. Stateful-first (skip view function, go directly to proof persistence)

Combine verification and persistence in a single `submitProof` that writes to storage on every capture.

- **Pro:** Simpler -- one spec instead of two
- **Con:** Every verification costs gas (~40k+ for storage writes). No economic model yet for who pays. Blocks the entire feature on solving the gas economics question.
- **Verdict:** Rejected. The view function delivers immediate value (trustless verification) with zero gas cost. Persistence can be layered on top.

## Consequences

### Positive

- Trustless verification available immediately -- anyone can call the view function to verify a capture without trusting the attester
- Zero gas cost for verification (view call)
- Minimal contract changes (one new mapping, one new function)
- Clean upgrade path to S2b.2 proof persistence
- OZ ECDSA protects against signature malleability from day one

### Negative

- Pre-computed hash means the contract cannot independently verify *what* was signed -- it only verifies *who* signed. A malicious caller could pass a fabricated hash. This is acceptable because the capture JSON contains both the hash inputs and the signature, so any verifier can recompute the hash off-chain before calling the contract.
- Existing registered devices (if any) will not have the `registeredDevices` reverse mapping populated. Requires either redeployment or a backfill function. At current stage (testnet), redeployment is acceptable.
- No replay protection at the contract level -- the same (hash, v, r, s) can be verified multiple times. This is fine for a view function but must be addressed in S2b.2.
