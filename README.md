# HardTrust

DePIN device identity and attestation system.

HardTrust enables physical devices (starting with Raspberry Pi) to cryptographically prove their identity and attest sensor readings on-chain. Devices sign data with secp256k1 keys, and an EVM smart contract verifies those signatures — creating a trustless bridge between hardware and blockchain.

## The Wire — Walking Skeleton

"The Wire" is the end-to-end walking skeleton proving the core value proposition:

> A device that is registered on-chain is **VERIFIED**. A device that is not registered is **UNVERIFIED**.

The full flow in one command:

```bash
just e2e-the-wire
# Expected output: The Wire gate: PASSED
```

What it does:
1. Starts a local Anvil chain
2. Deploys the `HardTrustRegistry` contract
3. Runs `device init` to print device identity (serial + address)
4. Runs `attester register` to register the device on-chain
5. Runs `device emit` to write a mock `reading.json`
6. Runs `attester verify` on the reading — expects **VERIFIED**
7. Runs `attester verify` on a fake reading — expects **UNVERIFIED**

## Architecture

```
contracts/       Solidity smart contract (Foundry) — HardTrustRegistry
device/          Rust binary — device identity and data emission
attester/        Rust binary — CLI for registration and verification
types/           Rust library — shared Reading struct and dev constants
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Foundry](https://getfoundry.sh/) (`forge`, `cast`, `anvil`)
- [just](https://github.com/casey/just) (task runner)
- [Node.js](https://nodejs.org/) (for `solhint`)

## Quick Start

```bash
# Build everything (contracts first, then Rust crates)
just build

# Run all tests
just test

# Run the full walking skeleton gate
just e2e-the-wire
```

## Development

```bash
# Lint (cargo fmt, clippy, forge fmt, solhint, aderyn)
just lint

# Full CI (lint + test)
just ci
```

One story per branch, one PR per story. Run `just ci` before every commit.
See [CLAUDE.md](CLAUDE.md) for the full development workflow.

## Repository Structure

```
hardtrust/
├── contracts/          # Solidity (Foundry project)
│   ├── src/            # Contract source
│   ├── test/           # Foundry tests
│   └── script/         # Deploy scripts
├── device/             # Rust binary — device CLI
├── attester/           # Rust binary — attester CLI
├── types/              # Rust library — shared types and dev constants
├── scripts/            # Shell scripts (e2e flows)
├── docs/
│   ├── adr/            # Architecture Decision Records
│   ├── specs/          # Feature specifications
│   └── stories/        # User stories
├── Cargo.toml          # Rust workspace
├── justfile            # Task runner
└── CLAUDE.md           # AI-assisted development rules
```

## Key Design Decisions

- **secp256k1/ECDSA** for device identity — EVM-native verification via `ecrecover`
- **Hybrid storage** — device registration on-chain, sensor data off-chain
- **Single registry contract** — handles both identity and attestation for MVP simplicity
- **Alloy** for Rust-to-EVM bindings (successor to ethers-rs)

See [docs/adr/](docs/adr/) for detailed rationale.

## CI

GitHub Actions runs on every push and PR to `main`:

| Job | What it checks |
|-----|----------------|
| **lint** | `cargo fmt`, `cargo clippy`, `forge fmt`, `solhint` |
| **test** | `cargo test`, `forge test` |

## License

TBD
