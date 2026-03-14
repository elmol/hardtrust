# S1a.3 — Verify Data from an Unregistered Device

## User Story

As anyone, I can see that data from an unregistered device is UNVERIFIED, so that I can distinguish untrusted sources from trusted ones.

## Context

This is the contrast case that completes The Wire. A reading.json with a non-registered address is verified against the registry and gets UNVERIFIED. Combined with S1a.2, this proves the core value proposition: registered devices are verified, unregistered ones are not.

## Acceptance Criteria

- [ ] Given a reading.json with an address that is NOT in the registry
- [ ] Running `attester verify --file reading.json` queries the registry and prints **UNVERIFIED**
- [ ] Both cases are demonstrable in a single terminal session: registered → VERIFIED, unregistered → UNVERIFIED
- [ ] Workspace builds with `cargo build --workspace`, tests pass with `cargo test --workspace` and `forge test`

## Out of Scope

- Tampered data (valid device but altered reading) → S1b
- Detailed error messages → S1b
- Automated e2e script → S1c
- CI pipeline changes → S1c

## Related ADRs

- ADR-0004: Single smart contract for registry + attestation

## Definition of Done

- VERIFIED for registered device, UNVERIFIED for unregistered — demonstrable side by side
- Tests cover both paths
- Full end-to-end flow runs: init → deploy → register → emit → verify (VERIFIED) + unregistered verify (UNVERIFIED)
- Workspace configured, `just build` and `just test` work
- **The Wire gate is met**
