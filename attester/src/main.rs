use alloy::{
    network::EthereumWallet,
    primitives::{keccak256, Address, FixedBytes},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use clap::{Parser, Subcommand};
use hardtrust_types::Reading;

sol!(
    #[sol(rpc)]
    HardTrustRegistry,
    "../contracts/out/HardTrustRegistry.sol/HardTrustRegistry.json"
);

#[derive(Parser)]
#[command(name = "attester")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Register a device on-chain
    Register {
        /// Device serial string
        #[arg(long)]
        serial: String,
        /// Device Ethereum address
        #[arg(long)]
        device_address: Address,
        /// Deployed HardTrustRegistry contract address
        #[arg(long)]
        contract: Address,
    },
    /// Verify a device reading against on-chain registration
    Verify {
        /// Path to reading.json file
        #[arg(long)]
        file: String,
        /// Deployed HardTrustRegistry contract address
        #[arg(long)]
        contract: Address,
    },
}

/// Check if a device is verified: on-chain address is non-zero and matches the reading address.
fn is_verified(on_chain_addr: Address, reading_addr: Address) -> bool {
    on_chain_addr != Address::ZERO && on_chain_addr == reading_addr
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
            let serial_hash = keccak256(serial.as_bytes());

            // Anvil account #1 private key
            let signer: PrivateKeySigner =
                "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
                    .parse()
                    .expect("valid private key");
            let wallet = EthereumWallet::from(signer);

            let provider = ProviderBuilder::new()
                .wallet(wallet)
                .connect_http("http://127.0.0.1:8545".parse().expect("valid URL"));

            let registry = HardTrustRegistry::new(contract, &provider);
            let serial_hash_bytes: FixedBytes<32> = serial_hash.into();
            let tx = registry
                .registerDevice(serial_hash_bytes, device_address)
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

            let reading_addr: Address = reading.address.parse().expect("invalid reading address");
            let serial_hash = keccak256(reading.serial.as_bytes());

            let provider = ProviderBuilder::new()
                .connect_http("http://127.0.0.1:8545".parse().expect("valid URL"));

            let registry = HardTrustRegistry::new(contract, &provider);
            let serial_hash_bytes: FixedBytes<32> = serial_hash.into();
            let result = registry
                .getDevice(serial_hash_bytes)
                .call()
                .await
                .expect("failed to query contract");

            if is_verified(result.deviceAddr, reading_addr) {
                println!("VERIFIED");
            } else {
                println!("UNVERIFIED");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_verified_true_when_addresses_match() {
        let addr: Address = "0x1234567890abcdef1234567890abcdef12345678"
            .parse()
            .unwrap();
        assert!(is_verified(addr, addr));
    }

    #[test]
    fn is_verified_false_when_addresses_differ() {
        let on_chain: Address = "0x1234567890abcdef1234567890abcdef12345678"
            .parse()
            .unwrap();
        let reading: Address = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
            .parse()
            .unwrap();
        assert!(!is_verified(on_chain, reading));
    }

    #[test]
    fn is_verified_false_when_on_chain_is_zero() {
        let reading: Address = "0x1234567890abcdef1234567890abcdef12345678"
            .parse()
            .unwrap();
        assert!(!is_verified(Address::ZERO, reading));
    }

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
