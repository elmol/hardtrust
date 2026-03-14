# Spec: S1a.1-V1 — HardTrustRegistry Contract

## Parent Story

S1a.1 — Register a Device and Confirm It On-Chain

## Goal

A minimal Solidity contract that lets an authorized attester register a device and lets anyone query whether a device is registered.

## What to Build

### `contracts/src/HardTrustRegistry.sol`

- Constructor takes one `address _attester` parameter, stores `msg.sender` as owner and `_attester` as authorized attester
- `registerDevice(bytes32 serialHash, address deviceAddr)` — requires `msg.sender == attester`, stores the mapping entry
- `getDevice(bytes32 serialHash) returns (address deviceAddr, address attester, uint256 attestedAt, bool active)` — returns zero values if not found
- `isAttester(address addr) returns (bool)` — returns whether the address is the authorized attester
- No events, no duplicate check, no owner management

### `contracts/test/HardTrustRegistry.t.sol`

- Attester can register a device and `getDevice` returns correct data
- Non-attester cannot register (expect revert)
- Unregistered serial hash returns zero values
- `isAttester` returns true for seeded attester, false for random address

### `contracts/script/Deploy.s.sol`

- Foundry deploy script using `vm.startBroadcast()` with Anvil account #0 private key (`0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`)
- Deploys HardTrustRegistry with Anvil account #1 as attester (`0x70997970C51812dc3A010C7d01b50e0d17dc79C8`)
- **Print the deployed address with `console.log("DEPLOYED:", address(registry))`** — this is used by the e2e script to extract the address

### Output files

- `contracts/src/HardTrustRegistry.sol`
- `contracts/test/HardTrustRegistry.t.sol`
- `contracts/script/Deploy.s.sol`

## What NOT to Build

- No Rust code
- No events, no duplicate check (intentional for walking skeleton — added in S1b)
- No dynamic attester management (addAttester/removeAttester)

## Validation

```bash
cd contracts && forge build && forge test -vvv
anvil &
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast
# Should print: DEPLOYED: 0x<contract-address>
```

## Handoff Prompt

```
Implement the HardTrustRegistry smart contract for HardTrust.

Read the spec at docs/specs/s1a.1-v1-registry-contract.spec.md first.

Output files:
- contracts/src/HardTrustRegistry.sol
- contracts/test/HardTrustRegistry.t.sol
- contracts/script/Deploy.s.sol

The deploy script must use console.log("DEPLOYED:", address(registry)) to print the
contract address — the e2e script depends on this format.

Respect the "What NOT to Build" section strictly — do not add anything beyond what is specified.
After implementation, run `forge build && forge test -vvv` to validate.
Branch: feat/s1a.1-register-device
```
