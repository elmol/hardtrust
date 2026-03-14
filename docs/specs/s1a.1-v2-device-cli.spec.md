# Spec: S1a.1-V2 — Device CLI with `init` Command

## Parent Story

S1a.1 — Register a Device and Confirm It On-Chain

## Goal

A Rust binary in `device/` with a single `init` subcommand that prints a hardcoded device identity.

## What to Build

### `device/` crate

- Add `device` as a workspace member in root `Cargo.toml`
- Dependencies: `clap` (with `derive` feature)
- Single subcommand: `device init`
- Output (exactly two lines to stdout):
  ```
  Serial: HARDCODED-001
  Address: 0x1234567890abcdef1234567890abcdef12345678
  ```
- Values are string constants in source code

### Test

- Test via `std::process::Command` capturing stdout: assert output contains expected serial and address strings

### Output files

- `device/Cargo.toml`
- `device/src/main.rs`

## What NOT to Build

- No `common/` crate dependency
- No key generation, no crypto
- No file I/O, no persistence
- No `emit` subcommand (that's S1a.2)

## Validation

```bash
cargo build -p device && cargo test -p device
cargo run --bin device -- init
# Expected:
# Serial: HARDCODED-001
# Address: 0x1234567890abcdef1234567890abcdef12345678
```

## Handoff Prompt

```
Create the device CLI crate for HardTrust.

Read the spec at docs/specs/s1a.1-v2-device-cli.spec.md first.

Output files:
- device/Cargo.toml
- device/src/main.rs

Add device/ as a new workspace member. Hardcoded values, one subcommand.
Respect the "What NOT to Build" section strictly — do not add anything beyond what is specified.
After implementation, run `cargo build -p device && cargo test -p device` to validate.
Branch: feat/s1a.1-register-device
```
