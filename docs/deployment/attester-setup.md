# Attester Setup — HardTrust v0.1.0

The `attester` binary registers devices on-chain and verifies signed readings. It requires
an Ethereum wallet key and access to a JSON-RPC endpoint.

**v0.1.0 scope:** Local Anvil only. Testnet/mainnet deployment is not covered in this release.

---

## Prerequisites

- Ubuntu 22.04+ (x86_64) or macOS with Apple Silicon (arm64)
- `curl` (pre-installed)
- [Foundry](https://getfoundry.sh/) — `anvil` and `forge`
- An Ethereum private key for the attester wallet

**Install Foundry:**
```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

Verify:
```bash
anvil --version
forge --version
```

---

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/elmol/hardtrust/main/install.sh | bash
```

Verify:
```bash
attester --help
```

Expected output:
```
HardTrust attester CLI — register and verify devices

Usage: attester <COMMAND>

Commands:
  register  Register a device on-chain
  verify    Verify a device reading against on-chain registration
```

---

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `HARDTRUST_PRIVATE_KEY` | **Yes** for `register` | — | Attester signing key, hex with `0x` prefix |
| `HARDTRUST_RPC_URL` | No | `http://127.0.0.1:8545` | Ethereum JSON-RPC endpoint |

`HARDTRUST_PRIVATE_KEY` is not required for `verify` — it is read-only.

**Security note:** Never commit `HARDTRUST_PRIVATE_KEY` to version control or store it in
shell history. For production use, inject it via a secrets manager.

---

## Local Anvil Setup (v0.1.0)

**Step 1 — Start Anvil** in a separate terminal and leave it running:
```bash
anvil
```

**Step 2 — Deploy the HardTrustRegistry contract** from the `hardtrust` repo root:
```bash
cd contracts
ATTESTER_ADDRESS=0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  forge script script/Deploy.s.sol \
  --broadcast \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

The output includes the deployed contract address:
```
DEPLOYED: 0x5FbDB2315678afecb367f032d93F642f64180aa3
```

Record this address — it is required as `--contract` in all subsequent commands.

**Note:** Anvil state is ephemeral. Restarting Anvil resets the chain — redeploy the
contract and re-register all devices.

---

## Registering a Device

Collect from the device operator:
- `Serial` from `device init` output
- `Address` from `device init` output

```bash
export HARDTRUST_PRIVATE_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"

attester register \
  --serial "100000004d01af60" \
  --device-address 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC \
  --contract 0x5FbDB2315678afecb367f032d93F642f64180aa3
```

**Expected output:**
```
Registered device. tx: 0xabc123...
```

---

## Verifying a Reading

```bash
attester verify \
  --file /path/to/reading.json \
  --contract 0x5FbDB2315678afecb367f032d93F642f64180aa3
```

**Expected output — registered device with valid signature:**
```
VERIFIED
```

**Expected output — unregistered device or tampered reading:**
```
UNVERIFIED
```

---

## Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| `HARDTRUST_PRIVATE_KEY env var is required` | `register` run without env var | `export HARDTRUST_PRIVATE_KEY="0x..."` |
| `invalid HARDTRUST_PRIVATE_KEY` | Key not valid hex or wrong format | Key must be `0x` + 64 hex chars (66 chars total) |
| `device already registered (serial hash: ...)` | Serial already registered on this chain | Each serial can only be registered once per contract deployment |
| `registration transaction failed: connection refused` | Anvil not running or wrong RPC URL | Start Anvil, check `HARDTRUST_RPC_URL` |
| `could not read reading file ...` | Path does not exist | Verify the path: `ls -la /path/to/reading.json` |
| `invalid reading JSON: missing field ...` | reading.json is incomplete | All 5 fields required: `serial`, `address`, `temperature`, `timestamp`, `signature` |
| `contract query failed` | RPC unreachable or wrong contract address | Check Anvil is running, verify contract address |
| `UNVERIFIED` (unexpected) | Key mismatch or tampered reading | Ensure device was not re-initialized after registration; verify reading.json was not modified |

---

## Typical Workflow

```bash
# Terminal 1 — leave running
anvil

# Terminal 2
export HARDTRUST_PRIVATE_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"

# Deploy (once per Anvil session)
cd contracts
ATTESTER_ADDRESS=0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  forge script script/Deploy.s.sol --broadcast --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
# Record CONTRACT_ADDRESS from output

# Register the device (once per device serial)
attester register \
  --serial "100000004d01af60" \
  --device-address 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC \
  --contract $CONTRACT_ADDRESS

# Verify a reading from the device
attester verify \
  --file ~/reading.json \
  --contract $CONTRACT_ADDRESS
```
