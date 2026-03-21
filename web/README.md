# HardTrust Web

React frontend for the `HardTrustRegistry` contract in `../contracts`.

## What it does

- Reads every registered device by querying the `DeviceRegistered` event and resolving each entry with `getDevice(...)`
- Connects an injected wallet such as MetaMask
- Lets the authorized attester submit `registerDevice(serialHash, deviceAddr)` directly from the browser
- Shows a dedicated "Become a verified device" section that links from the hero area

## Run it

• The local test path is:

  1. Start Anvil.

  anvil

  2. In another terminal, deploy the contract from contracts/script/Deploy.s.sol:

  cd ./terra-genesis/contracts
  ATTESTER_ADDRESS=0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  forge script script/Deploy.s.sol \
    --broadcast \
    --rpc-url http://127.0.0.1:8545 \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

  3. Create web/.env.

  cd ./terra-genesis/web
  Copy `.env.example` to `.env`:
  VITE_RPC_URL=http://127.0.0.1:8545
  VITE_CHAIN_ID=31337
  VITE_CONTRACT_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3

  4. Start the web app:

  npm install
  npm run dev

  5. In MetaMask:

  - Add network http://127.0.0.1:8545, chain ID 31337
  - Import the attester account:
      - Address: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
      - Private key: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d

  Then test both flows the web implements:

  - Read flow: page load should query DeviceRegistered logs and show registered devices.
  - Write flow: connect MetaMask, fill serial + device address, click Register device. The browser hashes the serial as keccak256(utf8(serial)) and calls registerDevice(...) as defined in web/src/App.jsx:252 and configured by web/src/
    contract.js:80.

  For a quick smoke test, use:

  - Serial: test-device-1
  - Device address: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC

## Default local setup

The defaults match your deployment docs:

- RPC: `http://127.0.0.1:8545`
- Chain ID: `31337`
- Example contract address: `0x5FbDB2315678afecb367f032d93F642f64180aa3`

If you use Anvil, connect MetaMask to the local chain and import the attester key for:

`0x70997970C51812dc3A010C7d01b50e0d17dc79C8`

Only that authorized attester account can submit registrations successfully.
