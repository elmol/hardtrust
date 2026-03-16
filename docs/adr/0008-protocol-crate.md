# ADR-0008 — Protocol Crate Naming

## Status

Accepted

## Context

The shared library crate defines the agreement between device and attester:
what a Reading is, how it's canonically hashed, and how it's signed/verified.
It was originally named `hardtrust-types`, then renamed to `hardtrust-core`.
Neither name communicates the crate's actual role.

## Decision

The shared library crate is named `hardtrust-protocol`. Internally it is organized
into modules by concern:

| Module | Contents |
|--------|----------|
| `domain.rs` | `Reading` struct (data model) |
| `crypto.rs` | `sign_reading`, `verify_reading`, `reading_prehash`, `public_key_to_address` |
| `error.rs` | `ProtocolError` enum |
| `dev_config.rs` | Dev-only constants (Anvil keys, addresses) |

All public items are re-exported from `lib.rs` so consumers use a flat API:
`use hardtrust_protocol::{Reading, sign_reading, ...}`.

## Alternatives Rejected

- **`hardtrust-core`** — too generic, tends to become a catch-all for anything
  that doesn't clearly belong elsewhere.
- **Multiple crates** (`hardtrust-crypto`, `hardtrust-domain`) — over-engineered
  at this scale. `sign_reading` and `verify_reading` are tightly coupled to
  `Reading` via `reading_prehash`; splitting them across crates adds dependency
  complexity with no benefit.
- **Move sign→device, verify→attester** — creates cross-dependency in tests
  (attester needs `sign_reading` for test fixtures).

## Consequences

- Both `device` and `attester` depend on `hardtrust-protocol`.
- Internal modules organize by concern without crate-level splitting.
- Future protocol additions (e.g., new message types) have a clear home.
