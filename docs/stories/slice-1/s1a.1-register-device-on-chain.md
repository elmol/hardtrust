# S1a.1 — Register a Device and Confirm It On-Chain

## User Story

As an Attester, I can register a device's identity on a local chain and confirm its registration, so that I know the device is part of the trusted registry.

## Context

This is the first horizontal slice of The Wire. It proves the registration pipeline works end-to-end: a device produces an identity, the attester registers it on-chain, and its existence can be confirmed. Everything is hardcoded — no real crypto.

## Acceptance Criteria

- [ ] Running `device init` prints a hardcoded serial and Ethereum address to stdout
- [ ] A HardTrustRegistry contract can be deployed to Anvil with a seeded attester
- [ ] Running `attester register` submits a registration transaction and prints the tx hash
- [ ] Querying the contract confirms the device is registered (verifiable via `cast call` or similar)
- [ ] The full sequence runs in one terminal session: Anvil → deploy → device init → attester register → query confirmation

## Out of Scope

- Data emission (reading.json) → S1a.2
- VERIFIED / UNVERIFIED output → S1a.2
- Real crypto (secp256k1, ECDSA) → S1b
- Duplicate registration check → S1b
- Error handling → S1b/S1c

## Related ADRs

- ADR-0001: Monorepo flat structure
- ADR-0003: Alloy for Rust-EVM bindings
- ADR-0004: Single smart contract for registry + attestation

## Definition of Done

- Device identity is created (hardcoded) and registered on-chain
- Registration is confirmed by querying the contract
- Foundry tests pass for the contract
- Cargo tests pass for device and attester
- The sequence is reproducible from a clean Anvil
