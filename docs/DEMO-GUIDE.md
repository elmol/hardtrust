# TerraGenesis Demo Guide

One-liner commands for recording a demo. Run all commands from the repo root.

---

## Step 0 — Load Environment

```bash
source .env.demo
```

> Edit `.env.demo` first. Fill `DEVICE_ADDR`, `SERIAL`, `CONTRACT_ADDR` as you go and re-source.

---

## Step 1 — RPi: Initialize Device

```bash
sshpass -p "$RPI_PASS" ssh $RPI_USER@$RPI_HOST "rm -f /home/pi/.hardtrust
/device.key"
sshpass -p "$RPI_PASS" ssh $RPI_USER@$RPI_HOST "device init"
```

> Copy `DEVICE_ADDR` and `SERIAL` into `.env.demo` → `source .env.demo`

---

## Step 2 — Local: Start Anvil (separate terminal)

```bash
source .env.demo && anvil
```

---

## Step 3 — Local: Deploy Contract

```bash
cd contracts && ATTESTER_ADDRESS=$ATTESTER_ADDRESS forge script script/Deploy.s.sol:Deploy --rpc-url $RPC_URL --private-key $ANVIL_KEY --broadcast && cd ..
```

> Copy `CONTRACT_ADDR` into `.env.demo` → `source .env.demo`

---

## Step 4 — Local: Register Device On-Chain

```bash
HARDTRUST_PRIVATE_KEY=$ANVIL_KEY HARDTRUST_RPC_URL=$RPC_URL ./target/release/attester register --serial $SERIAL --device-address $DEVICE_ADDR --contract $CONTRACT_ADDR
```

---

## Step 5 — RPi: Capture

```bash
sshpass -p "$RPI_PASS" ssh $RPI_USER@$RPI_HOST "device capture"
```

> `capture.json` is written to RPi's home. Captured files go to `~/capture-output/`.

---

## Step 6 — Transfer Files RPi to Local

```bash
mkdir -p ./demo-capture && sshpass -p "$RPI_PASS" scp $RPI_USER@$RPI_HOST:~/capture-output/* ./demo-capture/ && sshpass -p "$RPI_PASS" scp $RPI_USER@$RPI_HOST:~/capture.json ./demo-capture/
```

---

## Step 7 — Local: Verify Capture

```bash
HARDTRUST_RPC_URL=$RPC_URL ./target/release/attester verify --file ./demo-capture/capture.json --contract $CONTRACT_ADDR
```

> Expected: **VERIFIED (on-chain)**

---

## Step 8 — (Optional) Set Release Hashes + Re-verify

```bash
HARDTRUST_PRIVATE_KEY=$ANVIL_KEY HARDTRUST_RPC_URL=$RPC_URL ./target/release/attester set-release-hashes --script-hash $SCRIPT_HASH --binary-hash $BINARY_HASH --contract $CONTRACT_ADDR
```

```bash
HARDTRUST_RPC_URL=$RPC_URL ./target/release/attester verify --file ./demo-capture/capture.json --contract $CONTRACT_ADDR
```

> Environment should now show **MATCH (on-chain)**

---

## Demo Flow

```
source .env.demo              → load config
RPi: device init              → get address + serial → update .env.demo
Local: anvil                  → start testnet
Local: forge deploy           → get contract address → update .env.demo
Local: attester register      → register device on-chain
RPi: device capture           → capture image + sign
scp: RPi → Local              → transfer files
Local: attester verify        → VERIFIED (on-chain)
(opt) set-release-hashes      → register env hashes
(opt) attester verify         → MATCH (on-chain)
```
