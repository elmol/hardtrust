use chrono::Utc;
use clap::{Parser, Subcommand};
use hardtrust_types::{dev_config, Reading};

#[derive(Parser)]
#[command(name = "device", about = "HardTrust device CLI — identity and data emission")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print device serial and Ethereum address.
    ///
    /// Proves deterministic key derivation from the hardware serial number.
    /// The address printed here is the value to pass to `attester register`.
    Init,
    /// Emit a mock sensor reading to reading.json.
    ///
    /// Writes a JSON file containing the device serial, Ethereum address,
    /// temperature, timestamp, and a placeholder signature.
    /// This file is consumed by `attester verify`.
    Emit,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            println!("Serial: {}", dev_config::DEV_SERIAL);
            println!("Address: {}", dev_config::DEV_ADDRESS);
        }
        Command::Emit => {
            let reading = Reading {
                serial: dev_config::DEV_SERIAL.to_string(),
                address: dev_config::DEV_ADDRESS.to_string(),
                temperature: 42.0,
                timestamp: Utc::now().to_rfc3339(),
                signature: "0xFAKESIG".to_string(),
            };
            let json = serde_json::to_string_pretty(&reading).expect("failed to serialize reading");
            std::fs::write("reading.json", &json).expect("failed to write reading.json");
            println!("Wrote reading.json");
        }
    }
}

#[cfg(test)]
mod tests {
    use hardtrust_types::Reading;
    use std::process::Command;

    fn device_bin() -> std::path::PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.push("device");
        path
    }

    #[test]
    fn init_prints_serial_and_address() {
        let output = Command::new(device_bin())
            .args(["init"])
            .output()
            .expect("failed to run device binary");

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Serial: HARDCODED-001"));
        assert!(stdout.contains("Address: 0x1234567890abcdef1234567890abcdef12345678"));
    }

    #[test]
    fn emit_writes_valid_reading_json() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let output = Command::new(device_bin())
            .args(["emit"])
            .current_dir(dir.path())
            .output()
            .expect("failed to run device binary");

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Wrote reading.json"));

        let contents =
            std::fs::read_to_string(dir.path().join("reading.json")).expect("failed to read file");
        let reading: Reading = serde_json::from_str(&contents).expect("failed to parse JSON");
        assert_eq!(reading.serial, "HARDCODED-001");
        assert_eq!(
            reading.address,
            "0x1234567890abcdef1234567890abcdef12345678"
        );
        assert!((reading.temperature - 42.0).abs() < f64::EPSILON);
    }
}
