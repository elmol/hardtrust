# HardTrust — Architecture

## 1. System Overview

HardTrust is a DePIN device identity and attestation system. Physical devices (Raspberry Pi) generate cryptographic identities, trusted attesters register them on-chain, and anyone can verify device authenticity and data provenance.

```
                         +-----------+
                         |  Sepolia  |
                         |  Testnet  |
                         +-----+-----+
                               |
              +----------------+----------------+
              |                |                |
      +-------+------+ +------+------+ +-------+-------+
      |   attester/  | |  contracts/ | |    webapp/     |
      | CLI tool     | | Solidity    | | React          |
      | (Rust)       | | (Foundry)   | | (Slice 2+)    |
      +-------+------+ +------+------+ +-------+-------+
              |                                 |
              +--------+  common/  +------------+
              |        | (Rust lib)|
              |        +-----+----+
              |              |
      +-------+------+      |
      |   device/    +------+
      | RPi binary   |
      | (Rust)       |
      +--------------+
           |
     [Raspberry Pi]
      serial + keys
```

**Communication paths:**
- device → local JSON file (Slice 1) or API (Slice 2+)
- attester CLI → smart contract (on-chain tx via Alloy)
- webapp → smart contract (read registration status) + off-chain data store (read device readings)
- All Rust crates share crypto and types via the common library

## 2. Repository Structure

```
hardtrust/
├── Cargo.toml              # Workspace root (members: device, attester, common)
├── justfile                 # Task runner commands
├── contracts/              # Solidity (Foundry project, NOT a Cargo member)
│   ├── foundry.toml
│   ├── src/
│   ├── test/
│   ├── script/             # Deploy scripts
│   └── out/                # Build artifacts (ABI JSON consumed by common/)
├── device/                 # Rust binary — RPi package
│   ├── Cargo.toml
│   └── src/
├── attester/               # Rust binary — CLI registration tool
│   ├── Cargo.toml
│   └── src/
├── common/                 # Rust library — shared crypto, types, ABI bindings
│   ├── Cargo.toml
│   └── src/
├── webapp/                 # React app (Slice 2+)
│   └── ...
├── .github/
│   └── workflows/          # CI/CD pipelines
└── README.md
```

**Key boundaries:**
- `Cargo.toml` at root defines a workspace with members: `device`, `attester`, `common`
- `contracts/` is a standalone Foundry project — it is NOT part of the Cargo workspace
- The ABI bridge flows one direction: Foundry builds to `contracts/out/`, the `common` crate reads those artifacts at compile time via the Alloy `sol!` macro

## 3. Component Architecture

### 3.1 device/ (Rust binary)

**Responsibility:** Runs on the Raspberry Pi. Generates device identity, emits signed data readings.

**Capabilities:**
- Read hardware serial number from `/sys/firmware/devicetree/base/serial-number`
- Generate and persist a secp256k1 keypair (private key stored on filesystem)
- Detect existing keys and warn before overwriting (key persistence guard)
- Read CPU temperature from `/sys/class/thermal/thermal_zone0/temp`
- Sign data payloads (temperature + timestamp) with the device private key
- Output signed readings to a local JSON file (Slice 1) or POST to an API (Slice 2+)
- Emulation mode: simulated serial number and temperature for CI environments without physical hardware

**Inputs:** Hardware serial (from RPi), CPU temperature (from system), configuration (data directory, emission interval)

**Outputs:** Signed JSON data readings, device identity (serial + public key printed to console on init)

**Referenced stories:** S1a.1 (generate identity + register), S1a.2 (emit signed reading + verify), S3.4 (continuous emission)

### 3.2 attester/ (Rust binary)

**Responsibility:** CLI tool used by the Attester to register devices on-chain after physical verification.

**Capabilities:**
- Accept device serial + public key as CLI arguments
- Submit a registration transaction to the smart contract, signed with the Attester's private key
- Query device registration status from the contract
- Verify a device's emitted data by checking the signature against the on-chain public key
- Display transaction hash and confirmation status

**Inputs:** Device serial, device public key, Attester private key (for signing tx), RPC endpoint

**Outputs:** Transaction receipt (hash, status), verification result (registered/unregistered, signature valid/invalid)

**Referenced stories:** S1a.1 (register via CLI), S1a.2 (verify registered device), S1a.3 (verify unregistered device)

### 3.3 common/ (Rust library)

**Responsibility:** Shared code consumed by both `device/` and `attester/`. Single source of truth for cryptography, types, and contract interaction.

**Contents:**
- secp256k1 keypair generation and management
- ECDSA signing and verification
- Data payload types and serialization
- Smart contract ABI bindings (generated from Foundry output via Alloy `sol!` macro)
- Ethereum address derivation from public key

**Design constraint:** This crate has no CLI, no I/O, no side effects. Pure library.

### 3.4 contracts/ (Solidity / Foundry)

**Responsibility:** On-chain registry and attestation. Single smart contract for MVP.

See Section 4 for detailed smart contract design.

**Referenced stories:** All stories depend on this contract as the source of truth for device registration status.

### 3.5 webapp/ (React, Slice 2+)

**Responsibility:** Web application for attester registration (wallet-connected) and public device verification.

**Pages:**
- Attester view: wallet connection, device registration form, registration history (S2.1, S2.2, S2.5)
- Public verification view: device list with trust badges, live data updates (S2.3, S2.4)
- Demo view: side-by-side comparison of attested vs unattested device (S3.1, S3.2, S3.3)

**Interactions:**
- Reads registration status from the smart contract (via ethers.js or viem)
- Reads device data from off-chain store (local JSON in Slice 1 context, API in Slice 2+)
- Writes registration transactions through the Attester's connected wallet

**Deferred to feature specs:** Tech stack choices (React framework, wallet library, styling), API design, WebSocket vs polling for live data.

## 4. Smart Contract Design

### 4.1 Overview

A single Solidity contract handles both the attester registry and device registration. This keeps the MVP simple and avoids cross-contract complexity.

### 4.2 Data Structures

**Device record:**
- Serial hash (keccak256 of the hardware serial number)
- Device address (Ethereum address derived from the device's secp256k1 public key)
- Attester address (who registered this device)
- Attestation timestamp
- Active flag (for future revocation)

**Attester registry:**
- Mapping of Ethereum address to authorized status
- Attesters are pre-registered by the contract owner at deploy time

### 4.3 Functions

**Owner-only:**
- Add attester: authorize an Ethereum address as an attester
- Remove attester: revoke attester authorization
- *(Note: dynamic attester management (add/remove) is deferred — The Wire (S1a) uses a single immutable attester set at deploy time.)*

**Attester-only (caller must be an authorized attester):**
- Register device: accepts serial hash and device address, stores the device record with the caller as attester and current timestamp. Reverts if the serial hash is already registered or the caller is not an authorized attester. *(Note: duplicate serial hash check is deferred to S1b — intentionally omitted from the walking skeleton contract.)*

**Public (anyone can call):**
- Check device status: given a serial hash or device address, returns registration status and attestation details
- Check attester status: given an address, returns whether it is an authorized attester
- On-chain signature verification: verify that a data payload was signed by the private key corresponding to a registered device address (using ecrecover)

### 4.4 Access Control

- Contract owner: can manage the attester list. Set at deployment, non-transferable for MVP.
- Attesters: can register devices. Cannot modify other attesters or the owner.
- Devices: are registered entities, not callers. Devices do not transact on-chain directly.
- Public: read-only access to all registration data (transparency by design).

### 4.5 Design Notes

- Store hashes on-chain (keccak256 of serial), not raw serial numbers
- Device address (derived from public key) is stored directly since it is already a hash-derived value
- One registration per serial hash. Re-registration of the same serial reverts.
- Revocation deferred to Release 2 (story R2.3) but the active flag supports it
- No upgradability pattern for MVP. Contract is deployed as immutable.

## 5. Data Flow

### 5.1 Slice 1 — CLI End-to-End

```
[RPi]                    [Attester CLI]              [Smart Contract]          [Verifier CLI]
  |                           |                            |                        |
  |-- init: generate keys --->|                            |                        |
  |   (serial + pubkey        |                            |                        |
  |    printed to console)    |                            |                        |
  |                           |                            |                        |
  |   [manual: share serial + pubkey with attester]        |                        |
  |                           |                            |                        |
  |                           |-- register(serial, addr) ->|                        |
  |                           |<-- tx receipt -------------|                        |
  |                           |                            |                        |
  |-- emit: sign(temp+ts) -->|                             |                        |
  |   write to local JSON     |                            |                        |
  |                           |                            |                        |
  |                           |                            |<-- query status -------|
  |                           |                            |--- registered: yes --->|
  |                           |                            |                        |
  |                    [verifier reads local JSON, checks signature against on-chain pubkey]
  |                           |                            |                        |
  |                           |                            |          VERIFIED / UNVERIFIED
```

**Data at rest (Slice 1):**
- Device private key: filesystem on RPi (plain file, restricted permissions)
- Signed readings: local JSON file on RPi
- Registration records: on-chain (smart contract state)

### 5.2 Slice 2 — Web Portal

Same flow as Slice 1, with these changes:
- Attester registers devices through the webapp instead of CLI (wallet-connected transaction)
- Device readings are sent to a backend API instead of (or in addition to) local JSON
- The webapp reads device data from the API and registration status from the contract
- Live data updates via polling or WebSocket

### 5.3 Future — Decentralized Storage

- Device readings stored on IPFS instead of a centralized API
- Content hash of readings can be anchored on-chain for full provenance chain
- Deferred: no IPFS in the walking skeleton

### 5.4 Hybrid Storage Rationale

| Data type | Storage | Reason |
|-----------|---------|--------|
| Attester list | On-chain | Access control, must be tamper-proof |
| Device registration | On-chain | Source of truth for attestation status |
| Data readings (temperature) | Off-chain | High frequency, low value per reading, cost-prohibitive on-chain |
| Data signatures | Bundled with off-chain readings | Verified against on-chain identity when needed |

## 6. Cryptography

### 6.1 Algorithm Choice

**secp256k1 / ECDSA** for all device identity operations.

Rationale: EVM-native. The smart contract can verify signatures directly using `ecrecover`. No need for precompiles or custom verifiers. Aligns with DIMO's production pattern. Wallet tooling (MetaMask, Alloy) uses the same curve.

Note: The discovery document originally mentioned ed25519. This was revised to secp256k1 during architecture discussions to maintain EVM compatibility without additional complexity.

### 6.2 Key Generation Flow

1. Device runs init command
2. Read hardware serial from `/sys/firmware/devicetree/base/serial-number`
3. Generate a random secp256k1 private key (using OS entropy)
4. Derive the corresponding public key
5. Derive the Ethereum address from the public key (keccak256 of public key, take last 20 bytes)
6. Persist private key to filesystem (restricted permissions)
7. Print serial number and Ethereum address (device address) to console

The device address (Ethereum address derived from the public key) is the on-chain identity.

### 6.3 Signing Flow (Data Emission)

1. Device reads CPU temperature and current UTC timestamp
2. Construct a canonical payload (deterministic serialization of temperature + timestamp + serial)
3. Hash the payload (keccak256)
4. Sign the hash with the device's secp256k1 private key (ECDSA)
5. Output: payload + signature as a JSON structure

### 6.4 Verification Flow

1. Verifier obtains the signed JSON (from local file or API)
2. Extract the payload and signature
3. Reconstruct the payload hash using the same canonical serialization
4. Recover the signer address from the signature using ecrecover
5. Query the smart contract: is this recovered address registered as a device?
6. If registered: VERIFIED. If not: UNVERIFIED.

This verification can happen off-chain (in Rust using the common library) or on-chain (calling a contract function that uses ecrecover).

### 6.5 Key Storage Risk

Private keys are stored as plain files on the RPi filesystem. This is the weakest point in the security model. Accepted risk for the MVP given the educational context. Mitigations:
- Filesystem permissions (read-only by the device process)
- Physical attestation compensates: the Attester verifies the hardware in person
- Key regeneration is possible (requires re-attestation)

Future mitigations: encrypted key storage, secure element add-on (ATECC608A).

## 7. Development Workflow

### 7.1 Task Runner

All common operations go through `just` (justfile at repo root):

- `just test` — run cargo test (workspace) + forge test
- `just lint` — cargo fmt check + cargo clippy + forge fmt check
- `just integration` — start Anvil, deploy contracts, run integration tests
- `just e2e` — full end-to-end flow in emulation mode
- `just ci` — all of the above (mirrors the CI pipeline)
- `just release` — tag + changelog + push

### 7.2 CI/CD Pipeline (GitHub Actions)

**On every push / PR:**

1. Format and lint (fast gate): `cargo fmt --check`, `cargo clippy`, `forge fmt --check`
2. Unit tests: `cargo test --workspace`, `forge test`
3. Integration tests: spin up Anvil, deploy contracts, run integration test suite
4. E2E tests: full flow using device emulation mode

**On PRs only (additional gates):**

5. Security review: `anthropics/claude-code-security-review` GitHub Action
6. Code review: Claude Code Review (multi-agent, parallel analysis)
7. Solidity security: Trail of Bits `building-secure-contracts` skill

**Merge requirements:**
- All automated gates pass
- At least one human approval
- No direct pushes to main

### 7.3 Writer-Reviewer Cycle

The development pattern for each story:

```
Load spec → Claude Code implements → PR → Automated review → Human approval → Merge
                  |                              |
                  +---- test locally (tight) ----+
                  |                              |
                  +---- fix review findings -----+
```

- Writer (Claude Code) implements one story at a time, commits granularly
- Reviewer (Claude Code Review on PR) checks correctness, security, spec compliance
- Human reviews the PR after automated checks pass, approves merge
- AI never merges autonomously

### 7.4 Emulation Mode

The `device/` crate supports an emulation mode for CI and development without physical hardware:
- Simulated serial number (deterministic, based on config)
- Simulated CPU temperature (randomized within realistic range)
- Same signing and data emission logic as real mode
- Activated via CLI flag or environment variable

### 7.5 Local Development

- **Anvil** as the local Ethereum node
- Deploy contracts to Anvil with funded test accounts
- Attester CLI points to Anvil RPC
- Device runs in emulation mode
- Webapp connects to local Anvil

### 7.6 Recommended Skills and Tools

| Tool | Purpose |
|------|---------|
| Trail of Bits skills (`trailofbits/skills`) | Security analysis, especially `building-secure-contracts` for Solidity |
| `claude-code-security-review` | Automated security review on PRs |
| Claude Code Review (multi-agent) | Automated code review on PRs |
| Alloy | Rust library for EVM interaction |
| Foundry (forge, anvil, cast) | Solidity development, testing, local chain |

## 8. Architecture Decision Records (ADRs)

### 8.1 Purpose

ADRs capture significant technical and architectural decisions made during implementation. They preserve the **why** behind decisions so that future sessions (with Claude Code or humans) have context without re-discovering rationale.

This is especially important in AI-assisted development where each session starts without memory of prior decisions.

### 8.2 Location

```
hardtrust/
└── docs/
    └── adr/
        ├── 0001-monorepo-flat-structure.md
        ├── 0002-secp256k1-over-ed25519.md
        ├── 0003-alloy-for-evm-bindings.md
        └── ...
```

### 8.3 Template

Each ADR follows this format:

```markdown
# ADR-NNNN: [Title]

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-XXXX]

## Context
[What is the problem or decision that needs to be made?]

## Decision
[What was decided and why?]

## Consequences
[What are the positive and negative consequences of this decision?]

## Alternatives Considered
[What other options were evaluated and why were they rejected?]
```

### 8.4 When to Create an ADR

During implementation, Claude Code should create an ADR when:
- Choosing between two or more viable technical approaches
- Deviating from the architecture spec (with justification)
- Introducing a new dependency or tool
- Changing a data structure, API contract, or storage strategy
- Making a decision that would be non-obvious to a future reader

### 8.5 Implementation

- The HardTrust `CLAUDE.md` should include a rule: "When making a significant architectural or technical decision, create an ADR"
- A `/adr` skill can be created in HardTrust to generate ADRs from a consistent template
- ADRs are numbered sequentially (0001, 0002, ...) and never deleted — superseded ADRs are marked as such
- ADRs are committed alongside the code change that implements the decision

### 8.6 Seed ADRs

The following decisions from this architecture document should be captured as initial ADRs when the HardTrust repository is created:

| ADR | Decision |
|-----|----------|
| 0001 | Monorepo flat structure (device/, attester/, common/, contracts/, webapp/) |
| 0002 | secp256k1/ECDSA over ed25519 for EVM compatibility |
| 0003 | Alloy for Rust-EVM bindings over ethers-rs |
| 0004 | Single smart contract for registry + attestation |
| 0005 | Hybrid storage: registration on-chain, data off-chain |
| 0006 | Emulation mode for CI without physical hardware |

## 9. Release Strategy

### 9.1 Versioning

- **SemVer** (MAJOR.MINOR.PATCH) for the project as a whole
- Single version across the monorepo during MVP
- **Conventional commits** for all commit messages (enables automated changelog)

### 9.2 Environments

| Environment | Chain | Purpose | Deploy trigger |
|-------------|-------|---------|----------------|
| Local | Anvil | Development and testing | Manual (`just` commands) |
| Testnet | Sepolia | Staging, demos, workshops | Tagged release on main |

Production deployment is out of scope for the walking skeleton.

### 9.3 Deploy Flow

```
Feature branch → PR → Review + CI → Merge to main → Tag (vX.Y.Z) → CI builds → Deploy to Sepolia
```

Contract deployment to Sepolia uses Foundry deploy scripts. Webapp hosting decision deferred to feature specs.

### 9.4 Slice Delivery Sequence

Aligned with the story map:

1. **Slice 1 (CLI):** contracts + device + attester + common. Full end-to-end via terminal.
2. **Slice 2 (Portal):** webapp added. Attester registration and public verification move to web.
3. **Slice 3 (Polish):** demo view, continuous emission, attestation provenance display.

Each slice is a deployable increment. Slice 1 is the foundation that all subsequent slices build on without replacing.

## 10. Open Decisions

These items are deliberately deferred to feature specs or implementation phase.

| Decision | Deferred to | Notes |
|----------|-------------|-------|
| Exact JSON payload schema for signed readings | Feature spec for S1a.2 | Must be canonical and deterministic |
| Webapp framework and wallet library | Feature spec for S2.1 | React is decided; specific libraries are not |
| API design for Slice 2 data ingestion | Feature spec for S2.3/S2.4 | REST vs WebSocket, hosting, auth |
| Webapp hosting/deployment target | Feature spec for Slice 2 | Static hosting (Vercel, Netlify, etc.) |
| Data emission interval default | Feature spec for S3.4 | Story map suggests 30 seconds |
| Contract deploy script parameterization | Implementation of S1a.1 | How attester addresses are seeded |
| Revocation mechanism details | Feature spec for R2.3 | Data structure supports it; logic deferred |
| Device naming scheme | Feature spec for R2.2 | Human-readable names from public key hash |
| Off-chain data persistence for Slice 2 | Feature spec for Slice 2 | Local JSON sufficient for Slice 1 |
| Specific Rust crate versions | Implementation | Alloy, secp256k1 library, serde |
| Error handling strategy | Implementation | Follows Rust conventions per crate |
