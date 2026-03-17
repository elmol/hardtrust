use alloy::{
    network::EthereumWallet, primitives::Address, providers::ProviderBuilder,
    signers::local::PrivateKeySigner, sol,
};
use attester::{prepare_registration, verify_device, VerificationResult};
use clap::{Parser, Subcommand};
use hardtrust_protocol::{dev_config, Reading};

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
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
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
                .map_err(|_| "invalid attester private key")?;
            let wallet = EthereumWallet::from(signer);

            let provider = ProviderBuilder::new().wallet(wallet).connect_http(
                dev_config::DEV_RPC_URL
                    .parse()
                    .map_err(|_| "invalid RPC URL")?,
            );

            let registry = HardTrustRegistry::new(contract, &provider);
            let tx = registry
                .registerDevice(reg.serial_hash, device_address)
                .send()
                .await
                .map_err(|e| format!("registration transaction failed: {e}"))?
                .watch()
                .await
                .map_err(|e| format!("registration transaction failed: {e}"))?;

            println!("tx: {tx}");
        }
        Command::Verify { file, contract } => {
            let json = std::fs::read_to_string(&file)
                .map_err(|e| format!("could not read reading file {file}: {e}"))?;
            let reading: Reading =
                serde_json::from_str(&json).map_err(|e| format!("invalid reading JSON: {e}"))?;

            let reg = prepare_registration(&reading.serial);

            let provider = ProviderBuilder::new().connect_http(
                dev_config::DEV_RPC_URL
                    .parse()
                    .map_err(|_| "invalid RPC URL")?,
            );

            let registry = HardTrustRegistry::new(contract, &provider);
            let result = registry
                .getDevice(reg.serial_hash)
                .call()
                .await
                .map_err(|e| format!("contract query failed: {e}"))?;

            match verify_device(&reading, result.deviceAddr) {
                VerificationResult::Verified => println!("VERIFIED"),
                VerificationResult::Unverified(_) => println!("UNVERIFIED"),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as ProcessCommand;

    fn attester_bin() -> std::path::PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.push("attester");
        path
    }

    #[test]
    fn verify_with_nonexistent_file_prints_error_not_panic() {
        let output = ProcessCommand::new(attester_bin())
            .args([
                "verify",
                "--file",
                "/nonexistent/path/reading.json",
                "--contract",
                "0x0000000000000000000000000000000000000000",
            ])
            .output()
            .expect("failed to run attester binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn verify_with_bad_json_prints_error_not_panic() {
        let tmp = tempfile::NamedTempFile::new().expect("create temp file");
        std::fs::write(tmp.path(), "bad json").unwrap();

        let output = ProcessCommand::new(attester_bin())
            .args([
                "verify",
                "--file",
                tmp.path().to_str().unwrap(),
                "--contract",
                "0x0000000000000000000000000000000000000000",
            ])
            .output()
            .expect("failed to run attester binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
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

    #[test]
    #[ignore] // Requires Anvil + deployed contract
    fn register_duplicate_shows_human_error() {
        let contract = std::env::var("HARDTRUST_CONTRACT")
            .expect("HARDTRUST_CONTRACT env var must be set");
        let serial = format!("DUP-TEST-{}", std::process::id());
        let device_addr = "0x0000000000000000000000000000000000000042";

        // First registration — should succeed
        let output1 = ProcessCommand::new(attester_bin())
            .args([
                "register",
                "--serial", &serial,
                "--device-address", device_addr,
                "--contract", &contract,
            ])
            .output()
            .expect("failed to run attester register");
        assert!(output1.status.success(), "first register should succeed: {}",
            String::from_utf8_lossy(&output1.stderr));

        // Second registration — same serial, should fail with human-readable error
        let output2 = ProcessCommand::new(attester_bin())
            .args([
                "register",
                "--serial", &serial,
                "--device-address", device_addr,
                "--contract", &contract,
            ])
            .output()
            .expect("failed to run attester register");

        assert!(!output2.status.success());
        let stderr = String::from_utf8(output2.stderr).unwrap();
        assert!(
            stderr.contains("device already registered"),
            "expected human-readable duplicate error, got: {stderr}"
        );
    }

    #[test]
    #[ignore] // Requires Anvil + deployed contract
    fn register_success_shows_confirmation() {
        let contract = std::env::var("HARDTRUST_CONTRACT")
            .expect("HARDTRUST_CONTRACT env var must be set");
        let serial = format!("SUCCESS-TEST-{}", std::process::id());
        let device_addr = "0x0000000000000000000000000000000000000099";

        let output = ProcessCommand::new(attester_bin())
            .args([
                "register",
                "--serial", &serial,
                "--device-address", device_addr,
                "--contract", &contract,
            ])
            .output()
            .expect("failed to run attester register");

        assert!(output.status.success(), "register should succeed: {}",
            String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.contains("Registered device"),
            "expected 'Registered device' in output, got: {stdout}"
        );
        assert!(
            stdout.contains("tx:"),
            "expected 'tx:' in output, got: {stdout}"
        );
    }

    #[test]
    fn register_with_invalid_address_prints_error() {
        let output = ProcessCommand::new(attester_bin())
            .args([
                "register",
                "--serial",
                "TEST-001",
                "--device-address",
                "NOT_AN_ADDRESS",
                "--contract",
                "0x0000000000000000000000000000000000000000",
            ])
            .output()
            .expect("failed to run attester binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("address") || stderr.contains("invalid"),
            "expected error about address, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn register_with_invalid_contract_prints_error() {
        let output = ProcessCommand::new(attester_bin())
            .args([
                "register",
                "--serial",
                "TEST-001",
                "--device-address",
                "0x0000000000000000000000000000000000000001",
                "--contract",
                "GARBAGE",
            ])
            .output()
            .expect("failed to run attester binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("address") || stderr.contains("invalid") || stderr.contains("contract"),
            "expected error about address/contract, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn verify_with_missing_fields_prints_error() {
        let tmp = tempfile::NamedTempFile::new().expect("create temp file");
        std::fs::write(
            tmp.path(),
            r#"{"serial":"X","address":"Y","temperature":1.0}"#,
        )
        .unwrap();

        let output = ProcessCommand::new(attester_bin())
            .args([
                "verify",
                "--file",
                tmp.path().to_str().unwrap(),
                "--contract",
                "0x0000000000000000000000000000000000000000",
            ])
            .output()
            .expect("failed to run attester binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(
            stderr.contains("missing field"),
            "expected 'missing field' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn verify_with_empty_file_prints_error() {
        let tmp = tempfile::NamedTempFile::new().expect("create temp file");
        std::fs::write(tmp.path(), "").unwrap();

        let output = ProcessCommand::new(attester_bin())
            .args([
                "verify",
                "--file",
                tmp.path().to_str().unwrap(),
                "--contract",
                "0x0000000000000000000000000000000000000000",
            ])
            .output()
            .expect("failed to run attester binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }
}
