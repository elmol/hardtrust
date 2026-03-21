# TerraGenesis

> Proof-of-Physical-Data Layer for Microscopy

[![Status](https://img.shields.io/badge/status-hackathon%20prototype-orange)](https://github.com/biotexturas/terra-genesis)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![DeSci](https://img.shields.io/badge/domain-DeSci-green)](docs/proposal.md)

---

## Overview

**TerraGenesis** is a lightweight infrastructure layer that provides cryptographically verifiable provenance for biological data captured by physical microscopes.

Built on top of [TerraScope](https://github.com/biotexturas/TerraScope) — the DIY robotic microscope developed within the **Biotexturas / Landscapes of Opportunity** initiative — TerraGenesis explores how biological observations can become trustworthy digital artifacts anchored on-chain.

By linking physical microscopy with blockchain-based data provenance, TerraGenesis enables new infrastructure for **decentralized science (DeSci)**, open biotech, and biological data economies.

---

## Built on HardTrust

TerraGenesis is a specific application of [HardTrust](https://github.com/elmol/hardtrust), a generic **DePIN device identity and attestation framework**.

HardTrust enables any physical device to cryptographically prove its identity and attest data on-chain using secp256k1/ECDSA signatures verified by an EVM smart contract. TerraGenesis specializes this framework for **TerraScope microscopes**, turning biological image captures into verifiable on-chain records.

| Layer | Role |
|-------|------|
| **HardTrust** | Generic device identity, signing, and on-chain attestation |
| **TerraGenesis** | HardTrust applied to TerraScope microscopes for DeSci data provenance |

---

## The Problem

Scientific images and biological datasets are easily copied, altered, or detached from their physical source. In decentralized or community-driven research environments:

- There is no reliable way to prove **where** a biological observation originated.
- Datasets lack cryptographic provenance linking them to real instruments.
- Physical scientific infrastructure cannot easily integrate with **Web3** or decentralized data ecosystems.

---

## The Solution

TerraGenesis introduces a minimal system allowing biological images captured by TerraScope microscopes to be:

1. **Cryptographically hashed** at capture time
2. **Signed** by the capturing device
3. **Anchored on-chain** as an immutable proof of capture

This creates a verifiable record that a specific instrument generated a specific dataset at a specific moment in time.

> Rather than claiming absolute physical truth, TerraGenesis focuses on establishing **cryptographic provenance from authorized physical instruments**.

---

## Architecture (MVP)

```
┌─────────────────────────────────────────────────────────┐
│                    TerraGenesis MVP                      │
├─────────────┬──────────────┬─────────────┬──────────────┤
│  Device     │  Capture &   │  On-Chain   │  Verify      │
│  Registry   │  Sign        │  Proof      │  Anyone      │
│             │              │             │              │
│  Authorized │  Hash image  │  Device ID  │  Image not   │
│  TerraScope │  Sign locally│  Image hash │  altered ✓   │
│  identities │  by device   │  Timestamp  │  Registered  │
│             │              │             │  source ✓    │
└─────────────┴──────────────┴─────────────┴──────────────┘
```

See [docs/architecture.md](docs/architecture.md) for the full technical architecture.

---

## Configuration

| Env Var | Required | Default | Description |
|---------|----------|---------|-------------|
| `HARDTRUST_PRIVATE_KEY` | Yes (for `register`) | — | Attester signing key (hex, e.g. `0x...`) |
| `HARDTRUST_RPC_URL` | No | `http://127.0.0.1:8545` | Ethereum JSON-RPC endpoint |
| `TERRASCOPE_RESOLUTION` | No | `1920x1080` | Capture resolution |
| `TERRASCOPE_QUALITY` | No | `90` | JPEG quality (1-100) |

For local development with Anvil, the e2e script sets these automatically.

---

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

---

## Capture — Microscopy Data Provenance

TerraGenesis extends the walking skeleton with microscopy image capture:

```bash
# On a TerraScope RPi with camera installed:
device capture --output-dir ./output/

# Verify the capture:
attester verify --file capture.json --contract <address>
```

What happens:
1. `device capture` calls the TerraScope capture script (`/usr/local/lib/terrascope/capture.sh`)
2. The script takes a photo and generates metadata
3. Device hashes all files, signs the content hash, writes `capture.json`
4. `attester verify` checks the signature against the on-chain registry

---

## CI

GitHub Actions runs on every push and PR to `main`:

| Job | What it checks |
|-----|----------------|
| **lint** | `cargo fmt`, `cargo clippy`, `forge fmt`, `solhint` |
| **test** | `cargo test`, `forge test` |
| **e2e** | `just e2e-the-wire` — full end-to-end (readings + captures) |

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Foundry](https://getfoundry.sh/) (`forge`, `cast`, `anvil`)
- [just](https://github.com/casey/just) (task runner)
- [Node.js](https://nodejs.org/) (for `solhint`)

### Build & Run

```bash
# Build everything (contracts first, then Rust crates)
just build

# Run all tests
just test

# Run the full walking skeleton gate
just e2e-the-wire
```

### Development

```bash
# Lint (cargo fmt, clippy, forge fmt, solhint)
just lint

# Full CI (lint + test)
just ci
```

One story per branch, one PR per story. Run `just ci` before every commit.
See [CLAUDE.md](CLAUDE.md) for the full development workflow.

---

## Release & Installation

TerraGenesis ships pre-compiled binaries for Raspberry Pi (ARMv7 musl) and Ubuntu/macOS (x86_64 / arm64).
Releases are published at [github.com/biotexturas/terra-genesis/releases](https://github.com/biotexturas/terra-genesis/releases).

### Install on Raspberry Pi (device + capture script)

```bash
curl -fsSL https://raw.githubusercontent.com/biotexturas/terra-genesis/main/install-device.sh | bash
```

This installs both the `device` binary and the TerraScope capture script.
Full setup guide: [docs/deployment/device-setup.md](docs/deployment/device-setup.md)

### Install on Ubuntu / macOS (attester binary)

```bash
curl -fsSL https://raw.githubusercontent.com/biotexturas/terra-genesis/main/install-attester.sh | bash
```

Full setup guide: [docs/deployment/attester-setup.md](docs/deployment/attester-setup.md)

### Cutting a release

```bash
cargo release patch --dry-run   # preview
cargo release patch              # 0.1.0 → 0.1.1
```

Requires `cargo-release` (`cargo install cargo-release`). See [release.toml](release.toml).

---

## Integration with Landscapes of Opportunity

Within the **Landscapes of Opportunity** ecosystem, TerraGenesis acts as the **data trust layer** for the TerraScope DePIN network.

It enables:

- Verifiable biological datasets
- Reproducible community science
- Trusted data for AI and computational analysis
- Future digital assets derived from microbial ecosystems

This infrastructure connects real ecological observations with digital worlds and scientific experimentation.

---

## Hackathon Scope

During this hackathon we aim to prototype:

- [x] Device identity and signing (via HardTrust)
- [x] Hashing and data capture pipeline
- [x] Minimal on-chain proof of data provenance
- [x] Simple verification workflow

The prototype demonstrates how **open hardware scientific instruments** can produce verifiable on-chain data.

> *How can decentralized physical instruments generate trustworthy scientific data in open networks?*

See [docs/proposal.md](docs/proposal.md) for the full hackathon proposal.

---

## Repository Structure

```
terra-genesis/
├── contracts/          # Solidity (Foundry) — HardTrustRegistry
│   ├── src/
│   ├── test/
│   └── script/
├── device/             # Rust binary — device CLI (init, emit, capture)
├── attester/           # Rust binary — attester CLI (register, verify)
├── protocol/           # Rust library — shared protocol (Signable, crypto, types)
├── terrascope/         # TerraScope hardware adapter (capture script)
├── scripts/            # Shell scripts (build, e2e, mock-capture, version check)
├── docs/
│   ├── proposal.md     # Hackathon proposal
│   ├── architecture.md
│   ├── adr/            # Architecture Decision Records
│   ├── deployment/     # Operator setup guides
│   ├── specs/          # Feature specifications
│   └── stories/        # User stories
├── Cargo.toml          # Rust workspace
├── release.toml        # cargo-release configuration
├── install-device.sh   # RPi installer (device + capture script)
├── install-attester.sh # Ubuntu/macOS installer
├── justfile            # Task runner
├── CLAUDE.md           # AI-assisted development rules
└── README.md
```

---

## Key Design Decisions

- **secp256k1/ECDSA** for device identity — EVM-native verification via `ecrecover`
- **Hybrid storage** — device registration on-chain, sensor data off-chain
- **Single registry contract** — handles both identity and attestation for MVP simplicity
- **Alloy** for Rust-to-EVM bindings (successor to ethers-rs)

See [docs/adr/](docs/adr/) for detailed rationale.

---

## Contributing

This project is part of the **Biotexturas / Landscapes of Opportunity** initiative.
Contributions, feedback, and forks are welcome.

---

## License

MIT
