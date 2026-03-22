# TerraGenesis

> Proof-of-Physical-Data Layer for Microscopy

[![Status](https://img.shields.io/badge/status-hackathon%20prototype-orange)](https://github.com/biotexturas/terra-genesis)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![DeSci](https://img.shields.io/badge/domain-DeSci-green)](docs/proposal.md)

---

## Overview

**TerraGenesis** is the verification layer that turns DIY microscopes into trusted nodes in a decentralized scientific network.

Built on top of [TerraScope](https://github.com/biotexturas/TerraScope) — the open-source robotic microscope from the **Biotexturas / Landscapes of Opportunity** initiative — TerraGenesis lets biological observations become cryptographically verifiable digital artifacts anchored on-chain.

> Build, rent, or access a network of scientific instruments to explore Nature's microscopic creatures — with AI agents and friends alike.

The idea is simple: scientific infrastructure should be **cheap, accessible, and playful**. Open Science Hardware (OSH) like TerraScope makes instruments available to everyone. TerraGenesis makes the data they produce **trustworthy** — so that citizen scientists, AI research agents, and communities can all rely on the same verifiable observations.

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

Science is trapped behind institutional walls, credentialism, and expensive instrumentation. Most potential citizen scientists never get the chance to explore. Meanwhile, AI research agents are confined to digital spaces — reading papers, generating hypotheses, but unable to test them physically.

The challenge is twofold:

1. **Physical research infrastructure should be cheap and ubiquitous.** Open Science Hardware solves part of this, but these instruments are often unreliable and not accessible online for remote collaboration.

2. **Hypothesis-driven science ("Day Science") is only part of the picture.** Before a rigorous experiment, there must be an observation — often made while *playing* in unrelated contexts. This is [Night Science](https://www.sciencedirect.com/science/article/pii/S1097276518306208): serendipitous discovery in a playful environment. To give agents and citizens a chance to interact with Nature, scientific instruments should work more like **game consoles** than lab equipment.

A good laboratory provides both: access to instruments and a protective environment where play can happen. Access to such spaces is increasingly hard to find.

---

## The Solution

TerraGenesis provides the **minimal Web3 verification layer** needed to turn a TerraScope into a trusted node in a Decentralized Physical Infrastructure Network (DePIN):

1. **Register** — a TerraScope device gets an on-chain identity (secp256k1 keypair, serial hash)
2. **Capture & Sign** — the microscope takes a photo, hashes everything, signs with its private key
3. **Verify on-chain** — anyone can call a free view function to confirm the data came from a registered instrument, untampered

This creates a verifiable record that a specific instrument generated a specific dataset at a specific moment — the **genesis** of trusted scientific data from open hardware.

> Rather than claiming absolute physical truth, TerraGenesis establishes *cryptographic provenance from authorized physical instruments*.

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

## The Wire — Walking Skeleton

"The Wire" is the end-to-end walking skeleton proving the core value proposition:

> A registered device is **VERIFIED**. An unregistered device is **UNVERIFIED**. A tampered environment is **MISMATCH**.

The full flow in one command:

```bash
just e2e-the-wire
# Expected output: The Wire gate: PASSED (6 cases)
```

What it tests:
1. **Reading VERIFIED** — registered device emits signed reading → VERIFIED
2. **Reading UNVERIFIED** — fake device reading → UNVERIFIED
3. **Capture VERIFIED** — registered device capture with signed content hash → VERIFIED
4. **Capture UNVERIFIED** — fake capture → UNVERIFIED
5. **Environment MATCH** — capture with approved release hashes → MATCH (on-chain)
6. **Environment MISMATCH** — tampered capture script → MISMATCH (on-chain), signature still VERIFIED

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

## On-Chain Verification

TerraGenesis verifies captures trustlessly on-chain using a view function (free, no gas):

```bash
# Verify a capture against on-chain registry
attester verify --file capture.json --contract <address>
```

The `verifyCapture()` function in the smart contract:
- Recovers the signer from the ECDSA signature (using OpenZeppelin ECDSA for malleability protection)
- Checks if the signer is a registered device
- Optionally compares environment hashes (script + binary) against approved release hashes

### Environment Attestation

Each capture includes environment metadata:
- `script_hash` — SHA256 of the capture script
- `binary_hash` — SHA256 of the device binary
- `hw_serial` — Hardware serial number
- `camera_info` — Camera model from device tree

Approved release hashes can be set on-chain:

```bash
attester set-release-hashes \
  --script-hash <sha256> \
  --binary-hash <sha256> \
  --contract <address>
```

---

## Web Interface

TerraGenesis includes a browser-based portal for inspecting the device registry and verifying captures:

- **Registry Browser** — view all registered devices with serial hash, address, and attestation status
- **Device Registration** — register new TerraScope devices on-chain (attester wallet required)
- **Capture Verification** — upload a capture manifest to verify provenance and environment on-chain

### Run locally

```bash
cd web
npm install
npm run dev
```

Configure via `.env` (see `web/.env.example`):
- `VITE_CONTRACT_ADDRESS` — deployed HardTrustRegistry address
- `VITE_RPC_URL` — JSON-RPC endpoint (default: `http://127.0.0.1:8545`)
- `VITE_EXPECTED_CHAIN_ID` — expected chain ID (default: `31337`)

Requires MetaMask or any injected EVM wallet.

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

## Configuration

| Env Var | Required | Default | Description |
|---------|----------|---------|-------------|
| `HARDTRUST_PRIVATE_KEY` | Yes (for `register`) | — | Attester signing key (hex, e.g. `0x...`) |
| `HARDTRUST_RPC_URL` | No | `http://127.0.0.1:8545` | Ethereum JSON-RPC endpoint |
| `TERRASCOPE_RESOLUTION` | No | `1920x1080` | Capture resolution |
| `TERRASCOPE_QUALITY` | No | `90` | JPEG quality (1-100) |

For local development with Anvil, the e2e script sets these automatically.

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

## Vision

We envision a decentralized world where people and AI agents play games accessing a DePIN of scientific instruments to interact with Nature — biology in particular.

With cheap, hackable, open-source hardware available to everyone, **playful interactions with Nature become the substrate for serious scientific discovery.** How does biological intelligence implement ecosystem function? Can statistics emerge across many independent devices? Can AI agents learn to operate instruments from humans, learn intelligence from smart bacteria, and together unravel the secrets of sustainability?

TerraGenesis makes the invisible **visible and verifiable**, and turns any agent or citizen into a scientist, a maker, or a player.

> *Is intelligence, in the end, about sustainability and persistence in a challenging environment? Can bacteria teach us — and teach our agents?*

---

## Hackathon Scope

During this hackathon we prototyped:

- [x] Device identity and signing (secp256k1/ECDSA, on-chain registry)
- [x] Microscopy data capture pipeline (hash, sign, manifest)
- [x] On-chain verification — trustless, free, permissionless (`verifyCapture()`)
- [x] Environment attestation (script hash, binary hash, hardware serial)
- [x] Web-based registry portal and capture verification UI
- [x] End-to-end test suite (6 cases: readings, captures, environment match/mismatch)

The prototype demonstrates how **open hardware scientific instruments** can produce verifiable on-chain data — the foundation for a DePIN of scientific instruments accessible to citizens and AI agents alike.

---

## What's Next

TerraGenesis is a hackathon prototype. Here's what comes next:

| Feature | Description | Status |
|---------|-------------|--------|
| **On-chain proof persistence** | Store capture proofs permanently on-chain (not just verify) | Designed (S2b.2) |
| **Multi-device DePIN** | Connect multiple TerraScopes into a decentralized network | Next milestone |
| **AI agent access** | API endpoints so research agents can trigger captures and verify data remotely | Planned |
| **GameFi layer** | Turn scientific observation into playable experiences — biotic games with real microbes | Vision |
| **Cross-device statistics** | Aggregate observations from distributed instruments for population-level insights | Vision |
| **Decentralized storage** | IPFS / Arweave for raw microscopy images with on-chain anchoring | Vision |
| **Merkle prehash** | Trustless on-chain environment verification without storing individual hashes | Designed (Option B) |

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
├── web/                # Vue.js frontend — registry portal + capture verifier
│   ├── src/
│   ├── assets/
│   └── package.json
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
- **OpenZeppelin ECDSA** for on-chain signature verification — protects against s-malleability, v-validation, zero address
- **On-chain verification model** — trustless view function (Model B: pre-computed hash, no gas). See [ADR-0010](docs/adr/adr-0010-onchain-verification-model.md)

See [docs/adr/](docs/adr/) for detailed rationale.

---

## Contributing

This project is part of the **Biotexturas / Landscapes of Opportunity** initiative.
Contributions, feedback, and forks are welcome.

---

## License

MIT
