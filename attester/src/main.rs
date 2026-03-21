use alloy::{
    network::EthereumWallet, primitives::Address, providers::ProviderBuilder,
    signers::local::PrivateKeySigner, sol,
};
use attester::{
    classify_registration_error, prepare_registration, verify_device_data, RegistrationError,
    VerificationResult,
};
use clap::{Parser, Subcommand};
use hardtrust_protocol::{Capture, Reading};

/// Release hashes embedded at compile time.
/// In debug builds, this is empty (env check skipped gracefully).
/// In release builds, it contains the SHA256SUMS from the repo root.
#[cfg(debug_assertions)]
const EMBEDDED_RELEASE_HASHES: &str = "";
#[cfg(not(debug_assertions))]
const EMBEDDED_RELEASE_HASHES: &str = include_str!("../../SHA256SUMS");

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum DeviceData {
    Capture(Capture),
    Reading(Reading),
}

sol!(
    #[sol(rpc)]
    HardTrustRegistry,
    "../contracts/out/HardTrustRegistry.sol/HardTrustRegistry.json"
);

#[derive(Parser)]
#[command(
    name = "attester",
    about = "HardTrust attester CLI — register and verify devices",
    version
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
    /// Verify a device reading or capture against on-chain registration.
    ///
    /// Reads a reading.json or capture.json file, auto-detects the format,
    /// queries the registry, and prints VERIFIED or UNVERIFIED.
    /// For captures, checks environment against embedded release hashes by default.
    Verify {
        /// Path to the reading.json or capture.json file
        #[arg(long)]
        file: String,
        /// Deployed HardTrustRegistry contract address
        #[arg(long)]
        contract: Address,
        /// Override embedded release hashes with a custom SHA256SUMS file
        #[arg(long)]
        release_hashes: Option<String>,
        /// Skip environment verification entirely
        #[arg(long, default_value_t = false)]
        skip_env_check: bool,
    },
}

fn env_rpc_url() -> Result<url::Url, Box<dyn std::error::Error>> {
    let raw =
        std::env::var("HARDTRUST_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
    raw.parse()
        .map_err(|_| format!("invalid RPC URL: {raw}").into())
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

            let private_key_hex = std::env::var("HARDTRUST_PRIVATE_KEY")
                .map_err(|_| "HARDTRUST_PRIVATE_KEY env var is required. Set it to the attester's hex-encoded private key.")?;
            let signer: PrivateKeySigner = private_key_hex.parse().map_err(|_| {
                "invalid HARDTRUST_PRIVATE_KEY — must be a hex-encoded private key (e.g. 0x...)"
            })?;
            let wallet = EthereumWallet::from(signer);

            let provider = ProviderBuilder::new()
                .wallet(wallet)
                .connect_http(env_rpc_url()?);

            let registry = HardTrustRegistry::new(contract, &provider);
            let tx = registry
                .registerDevice(reg.serial_hash, device_address)
                .send()
                .await
                .map_err(|e| {
                    match classify_registration_error(
                        &format!("{e}"),
                        &format!("{}", reg.serial_hash),
                    ) {
                        RegistrationError::AlreadyRegistered { serial_hash } => {
                            format!("device already registered (serial hash: {serial_hash})")
                        }
                        RegistrationError::TransactionFailed(msg) => {
                            format!("registration transaction failed: {msg}")
                        }
                    }
                })?
                .watch()
                .await
                .map_err(|e| format!("registration transaction failed: {e}"))?;

            println!("Registered device. tx: {tx}");
        }
        Command::Verify {
            file,
            contract,
            release_hashes,
            skip_env_check,
        } => {
            let json = std::fs::read_to_string(&file)
                .map_err(|e| format!("could not read file {file}: {e}"))?;

            let data: DeviceData = serde_json::from_str(&json)
                .map_err(|e| format!("invalid JSON (expected reading or capture): {e}"))?;

            let (serial, verification) = match &data {
                DeviceData::Capture(c) => {
                    let reg = prepare_registration(&c.serial);
                    let provider = ProviderBuilder::new().connect_http(env_rpc_url()?);
                    let registry = HardTrustRegistry::new(contract, &provider);
                    let result = registry
                        .getDevice(reg.serial_hash)
                        .call()
                        .await
                        .map_err(|e| format!("contract query failed: {e}"))?;
                    (c.serial.clone(), verify_device_data(c, result.deviceAddr))
                }
                DeviceData::Reading(r) => {
                    let reg = prepare_registration(&r.serial);
                    let provider = ProviderBuilder::new().connect_http(env_rpc_url()?);
                    let registry = HardTrustRegistry::new(contract, &provider);
                    let result = registry
                        .getDevice(reg.serial_hash)
                        .call()
                        .await
                        .map_err(|e| format!("contract query failed: {e}"))?;
                    (r.serial.clone(), verify_device_data(r, result.deviceAddr))
                }
            };

            let _ = serial; // used in future logging
            match verification {
                VerificationResult::Verified => println!("VERIFIED"),
                VerificationResult::Unverified(_) => println!("UNVERIFIED"),
            }

            // Environment verification for captures
            if let DeviceData::Capture(c) = &data {
                if !skip_env_check {
                    let hashes_content = if let Some(ref path) = release_hashes {
                        std::fs::read_to_string(path).map_err(|e| {
                            format!("could not read release hashes file {path}: {e}")
                        })?
                    } else {
                        EMBEDDED_RELEASE_HASHES.to_string()
                    };

                    if hashes_content.trim().is_empty()
                        || hashes_content.trim().starts_with('#')
                            && hashes_content.lines().count() <= 1
                    {
                        eprintln!(
                            "Warning: no release hashes available, skipping environment check"
                        );
                    } else {
                        let known = parse_hashes_content(&hashes_content);
                        println!("Environment:");
                        print_env_match(
                            "script_hash",
                            &c.environment.script_hash,
                            find_hash(&known, "capture.sh"),
                        );
                        print_env_match(
                            "binary_hash",
                            &c.environment.binary_hash,
                            find_hash(&known, "device"),
                        );
                        println!("  hw_serial:    {}", c.environment.hw_serial);
                        println!("  camera_info:  {}", c.environment.camera_info);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Parse SHA256SUMS content into (hash, filename) pairs.
fn parse_hashes_content(contents: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Format: "sha256:hex  filename" or "sha256:hex filename"
        if let Some((hash, name)) = line.split_once(|c: char| c.is_whitespace()) {
            entries.push((hash.trim().to_string(), name.trim().to_string()));
        }
    }
    entries
}

/// Find a hash in release entries by partial filename match.
fn find_hash<'a>(entries: &'a [(String, String)], needle: &str) -> Option<&'a str> {
    entries
        .iter()
        .find(|(_, name)| name.contains(needle))
        .map(|(hash, _)| hash.as_str())
}

/// Print a MATCH/MISMATCH line for an environment field.
fn print_env_match(field: &str, actual: &str, expected: Option<&str>) {
    match expected {
        Some(exp) if exp == actual => {
            println!("  {field}:  {actual} → MATCH");
        }
        Some(exp) => {
            println!("  {field}:  {actual} → MISMATCH (expected {exp})");
        }
        None => {
            println!("  {field}:  {actual} (no reference hash)");
        }
    }
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
            stderr.contains("missing field") || stderr.contains("did not match any variant"),
            "expected parse error on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn register_without_private_key_env_shows_clear_error() {
        let output = ProcessCommand::new(attester_bin())
            .env_remove("HARDTRUST_PRIVATE_KEY")
            .args([
                "register",
                "--serial",
                "TEST-001",
                "--device-address",
                "0x0000000000000000000000000000000000000001",
                "--contract",
                "0x0000000000000000000000000000000000000001",
            ])
            .output()
            .expect("failed to run attester binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("HARDTRUST_PRIVATE_KEY"),
            "expected error mentioning HARDTRUST_PRIVATE_KEY, got: {stderr}"
        );
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

    #[test]
    fn verify_capture_with_bad_json_prints_error() {
        let tmp = tempfile::NamedTempFile::new().expect("create temp file");
        // Has content_hash so detected as Capture, but invalid JSON structure
        std::fs::write(tmp.path(), r#"{"content_hash": "sha256:abc"}"#).unwrap();

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
    fn verify_capture_with_missing_fields_prints_error() {
        let tmp = tempfile::NamedTempFile::new().expect("create temp file");
        std::fs::write(
            tmp.path(),
            r#"{"serial":"X","address":"Y","content_hash":"sha256:abc","timestamp":"Z"}"#,
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
            stderr.contains("missing field") || stderr.contains("did not match any variant"),
            "expected parse error on stderr, got: {stderr}"
        );
    }
}
