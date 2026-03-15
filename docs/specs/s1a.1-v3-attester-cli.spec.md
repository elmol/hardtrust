# Spec: S1a.1-V3 — Attester CLI with `register` Command

## Parent Story

S1a.1 — Register a Device and Confirm It On-Chain

## Goal

A Rust binary in `attester/` with a single `register` subcommand that submits a device registration transaction on-chain.

## What to Build

### `attester/` crate

- Add `attester` as a workspace member in root `Cargo.toml`
- Dependencies in `attester/Cargo.toml`:
  ```toml
  [dependencies]
  clap = { version = "4", features = ["derive"] }
  alloy = { version = "1", features = ["full"] }
  tokio = { version = "1", features = ["full"] }
  ```
  Note: use `alloy` with `full` feature for the walking skeleton. Trim in a later slice.
- **No `common/` crate.** ABI binding via `sol!` macro directly in `attester/`:
  ```rust
  alloy::sol!(
      #[sol(rpc)]
      HardTrustRegistry,
      "../contracts/out/HardTrustRegistry.sol/HardTrustRegistry.json"
  );
  ```
- **Build dependency:** `forge build` must run before `cargo build -p attester` (the sol! macro reads from `contracts/out/`)
- ABI bindings will move to `common/` in a later slice (see architecture Section 3.3). For now, keep them in `attester/`.

### `attester register` subcommand

```
attester register --serial <SERIAL> --device-address <ADDRESS> --contract <CONTRACT_ADDRESS>
```

Behavior:
1. Connect to `http://127.0.0.1:8545` (hardcoded RPC)
2. Use Anvil account #1 private key as signer (hardcoded: `0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d`)
3. Compute `keccak256(serial.as_bytes())` using `alloy::primitives::keccak256` — this hashes the UTF-8 bytes of the serial string
4. Call `registerDevice(serialHash, deviceAddress)` on the contract
5. Print: `tx: 0x...`

Use `#[tokio::main]` on the binary entry point (Alloy is async).

### Test

- Integration test requiring Anvil + deployed contract, marked `#[ignore]`

### Output files

- `attester/Cargo.toml`
- `attester/src/main.rs`

## What NOT to Build

- No configurable RPC URL or private key (hardcoded for The Wire)
- No `verify` or `query` subcommand (that's S1a.2)
- No `common/` crate
- No error handling beyond `.expect()` with descriptive messages

## Validation

```bash
# Prerequisites: Anvil running, contract deployed (from V1)
cd contracts && forge build && cd ..
cargo build -p attester

cargo run --bin attester -- register \
  --serial HARDCODED-001 \
  --device-address 0x1234567890abcdef1234567890abcdef12345678 \
  --contract <CONTRACT_ADDRESS>
# Expected: prints "tx: 0x..."

# Confirm registration on-chain
SERIAL_HASH=$(cast keccak "HARDCODED-001")
cast call <CONTRACT_ADDRESS> "getDevice(bytes32)" $SERIAL_HASH --rpc-url http://127.0.0.1:8545
# Expected: returns non-zero deviceAddr
```

## Handoff Prompt

```
Create the attester CLI crate for HardTrust.

Read the spec at docs/specs/s1a.1-v3-attester-cli.spec.md first.

Output files:
- attester/Cargo.toml
- attester/src/main.rs

IMPORTANT: Run `cd contracts && forge build` before `cargo build -p attester`.
The sol! macro reads the ABI from contracts/out/ which only exists after forge build.

No common/ crate. Use Alloy sol! macro directly in attester/.
Respect the "What NOT to Build" section strictly — do not add anything beyond what is specified.
After implementation, start Anvil, deploy the contract, and run the register command.
Branch: feat/s1a.1-register-device (continue on existing branch)
```
