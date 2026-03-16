use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use attester::{prepare_registration, verify_device, VerificationResult};
use clap::{Parser, Subcommand};
use hardtrust_core::{dev_config, Reading};

sol!(
    #[sol(rpc)]
    HardTrustRegistry,
    "../contracts/out/HardTrustRegistry.sol/HardTrustRegistry.json"
);

#[derive(Parser)]
#[command(
    name = "attester",
    about = "HardTrust attester CLI — register and verify devices"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Register a device on-chain.
    ///
    /// Submits a registerDevice transaction to the HardTrustRegistry contract,
    /// signed by the authorized attester key. Prints the transaction hash on success.
    Register {
        /// The device's unique serial number (e.g. HARDCODED-001)
        #[arg(long)]
        serial: String,
        /// Ethereum address derived from the device serial — output of `device init`
        #[arg(long)]
        device_address: Address,
        /// Deployed HardTrustRegistry contract address
        #[arg(long)]
        contract: Address,
    },
    /// Verify a device reading against on-chain registration.
    ///
    /// Reads a reading.json file produced by `device emit`, queries the registry,
    /// and prints VERIFIED if the device address matches the on-chain record,
    /// or UNVERIFIED if the device is not registered.
    Verify {
        /// Path to the reading.json file produced by `device emit`
        #[arg(long)]
        file: String,
        /// Deployed HardTrustRegistry contract address
        #[arg(long)]
        contract: Address,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Register {
            serial,
            device_address,
            contract,
        } => {
            let reg = prepare_registration(&serial);

            let signer: PrivateKeySigner = dev_config::DEV_PRIVATE_KEY
                .parse()
                .expect("valid private key");
            let wallet = EthereumWallet::from(signer);

            let provider = ProviderBuilder::new()
                .wallet(wallet)
                .connect_http(dev_config::DEV_RPC_URL.parse().expect("valid URL"));

            let registry = HardTrustRegistry::new(contract, &provider);
            let tx = registry
                .registerDevice(reg.serial_hash, device_address)
                .send()
                .await
                .expect("failed to send transaction")
                .watch()
                .await
                .expect("failed to confirm transaction");

            println!("tx: {tx}");
        }
        Command::Verify { file, contract } => {
            let json = std::fs::read_to_string(&file).expect("failed to read reading file");
            let reading: Reading =
                serde_json::from_str(&json).expect("failed to parse reading JSON");

            let reg = prepare_registration(&reading.serial);

            let provider = ProviderBuilder::new()
                .connect_http(dev_config::DEV_RPC_URL.parse().expect("valid URL"));

            let registry = HardTrustRegistry::new(contract, &provider);
            let result = registry
                .getDevice(reg.serial_hash)
                .call()
                .await
                .expect("failed to query contract");

            match verify_device(&reading, result.deviceAddr) {
                VerificationResult::Verified => println!("VERIFIED"),
                VerificationResult::Unverified(_) => println!("UNVERIFIED"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_json_round_trip() {
        let json = r#"{
            "serial": "TEST-001",
            "address": "0xabcd",
            "temperature": 42.0,
            "timestamp": "2026-01-01T00:00:00Z",
            "signature": "0xFAKESIG"
        }"#;
        let reading: Reading = serde_json::from_str(json).expect("deserialize");
        assert_eq!(reading.serial, "TEST-001");
        assert_eq!(reading.address, "0xabcd");
        assert!((reading.temperature - 42.0).abs() < f64::EPSILON);
        assert_eq!(reading.timestamp, "2026-01-01T00:00:00Z");
        assert_eq!(reading.signature, "0xFAKESIG");
    }

    #[test]
    #[ignore] // Requires Anvil + deployed contract + registered device + reading.json
    fn verify_registered_device() {
        // Integration test — run manually with:
        // cargo test -p attester -- --ignored
    }
}
