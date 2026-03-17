# Device Setup — HardTrust v0.1.0

The `device` binary runs on a Raspberry Pi and is responsible for generating the device's
cryptographic identity and emitting signed temperature readings.

---

## Prerequisites

- Raspberry Pi (any model)
- Raspbian Bullseye 32-bit (ARMv7) — tested on Raspbian GNU/Linux 11
- `curl` (pre-installed on Raspbian)
- Internet connection for the initial download

No blockchain access or Ethereum node is required — the device binary works fully offline.

---

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/elmol/hardtrust/main/install.sh | bash
```

Verify:

```bash
device --help
```

Expected output:
```
HardTrust device CLI — generate key and emit readings

Usage: device <COMMAND>

Commands:
  init  Initialize device: generate a secp256k1 key pair
  emit  Emit a signed temperature reading
```

---

## Initialization

Run once per device. Generates a secp256k1 private key and derives the device's Ethereum address.

```bash
device init
```

**Output on a Raspberry Pi with hardware serial:**
```
Serial:  100000004d01af60
Address: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
```

**Output on a machine without hardware serial (emulated):**
```
Serial:  EMULATED-raspberrypi
Address: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
```

**Record the Serial and Address values.** You will need both when registering the device
with the attester.

The key is stored at `~/.hardtrust/device.key` with permissions `0600`. It is a
32-byte secp256k1 private key in hex encoding.

**Key backup warning:** The private key is not recoverable if lost. If `device.key` is
deleted, you will need to run `device init` again and re-register the device with a
new identity. The old on-chain registration becomes orphaned.

If the device is already initialized:
```
Device already initialized. Delete /home/pi/.hardtrust/device.key to regenerate.
```

To start fresh (destroys old identity — irreversible):
```bash
rm ~/.hardtrust/device.key
device init
```

---

## Emitting a Reading

```bash
device emit
```

**With a real thermal sensor:**
```
Wrote reading.json
```

**Without a thermal sensor (emulated):**
```
Wrote reading.json [EMULATED temperature]
```

This writes `reading.json` in the current directory. Example:

```json
{
  "serial": "100000004d01af60",
  "address": "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
  "temperature": 47.234,
  "timestamp": "2026-03-17T14:32:01Z",
  "signature": "0x1b3d22c..."
}
```

Transfer this file to the attester machine for on-chain registration and verification.

---

## Configuration

No environment variables are required. The binary uses `HOME` to locate `~/.hardtrust/device.key`.

| Path | Description |
|------|-------------|
| `~/.hardtrust/device.key` | secp256k1 private key, hex, permissions 0600 |
| `./reading.json` | Signed reading, written to current directory by `device emit` |

---

## Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| `Device already initialized. Delete ... to regenerate.` | Key file exists | Delete key file and re-run `init` only if you intend to create a new identity |
| `Device not initialized. Run 'device init' first.` | Ran `emit` before `init` | Run `device init` first |
| `Error: device.key contains invalid key data` | Key file corrupted | Delete `~/.hardtrust/device.key` and run `device init` |
| `Error: could not create ~/.hardtrust directory` | A file named `.hardtrust` exists instead of a directory | `ls -la ~/` to check, remove the file, retry |
| `Error: could not write device key` | Filesystem read-only or home not writable | `touch ~/test && rm ~/test` to check writability |
| `Wrote reading.json [EMULATED temperature]` | No thermal sensor found at `/sys/thermal_zone0/temp` | Informational only — reading is still valid and can be verified |

---

## Typical Workflow

```bash
# Step 1 — once: generate device identity
device init
# Record Serial and Address

# Step 2 — recurring: emit a signed reading
cd /some/working/directory
device emit
# reading.json is written to current directory

# Step 3 — transfer reading.json to attester
scp reading.json user@attester-host:~/
```
