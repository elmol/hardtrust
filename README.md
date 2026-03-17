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
5. Runs `device emit` to write a signed `reading.json` (with real CPU temp or emulated)
6. Runs `attester verify` on the reading — expects **VERIFIED**
7. Runs `attester verify` on a fake reading — expects **UNVERIFIED**

## Configuration

The attester binary reads configuration from environment variables:

| Env Var | Required | Default | Description |
|---------|----------|---------|-------------|
| `HARDTRUST_PRIVATE_KEY` | Yes (for `register`) | — | Attester signing key (hex-encoded, e.g. `0x...`) |
| `HARDTRUST_RPC_URL` | No | `http://127.0.0.1:8545` | Ethereum JSON-RPC endpoint |

For local development with Anvil, the e2e script sets these automatically.

## Architecture

```
contracts/       Solidity smart contract (Foundry) — HardTrustRegistry
device/          Rust binary — device identity and data emission
attester/        Rust binary — CLI for registration and verification
protocol/        Rust library — shared protocol types, crypto, and error handling
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Foundry](https://getfoundry.sh/) (`forge`, `cast`, `anvil`)
- [just](https://github.com/casey/just) (task runner)
- [Node.js](https://nodejs.org/) (for `solhint`)
- [aderyn](https://github.com/Cyfrin/aderyn) (optional, local-only — Solidity static analysis)

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
├── protocol/           # Rust library — shared protocol (Reading, crypto, errors)
├── scripts/            # Shell scripts (build, e2e, version check)
├── docs/
│   ├── adr/            # Architecture Decision Records
│   ├── deployment/     # Operator setup guides (device-setup.md, attester-setup.md)
│   ├── specs/          # Feature specifications
│   └── stories/        # User stories
├── Cargo.toml          # Rust workspace
├── release.toml        # cargo-release configuration
├── install-device.sh   # RPi installer (curl | bash)
├── install-attester.sh # Ubuntu/macOS installer (curl | bash)
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
| **e2e** | `just e2e-the-wire` — full end-to-end walking skeleton |

## Release & Installation

HardTrust ships pre-compiled binaries for Raspberry Pi (ARMv7 musl) and Ubuntu/macOS (x86_64 / arm64).
Releases are published at [github.com/elmol/hardtrust/releases](https://github.com/elmol/hardtrust/releases).

### Install on Raspberry Pi (device binary)

```bash
curl -fsSL https://raw.githubusercontent.com/elmol/hardtrust/main/install-device.sh | bash
```

Full setup guide: [docs/deployment/device-setup.md](docs/deployment/device-setup.md)

### Install on Ubuntu / macOS (attester binary)

```bash
curl -fsSL https://raw.githubusercontent.com/elmol/hardtrust/main/install-attester.sh | bash
```

Full setup guide: [docs/deployment/attester-setup.md](docs/deployment/attester-setup.md)

### Cutting a release

```bash
# Dry run first — see what will happen
cargo release patch --dry-run

# Cut the release (bumps Cargo.toml, commits, tags, pushes → CI builds binaries)
cargo release patch    # 0.1.0 → 0.1.1
cargo release minor    # 0.1.0 → 0.2.0
```

Requires `cargo-release` (`cargo install cargo-release`). See [release.toml](release.toml) for configuration.

## License

TBD
