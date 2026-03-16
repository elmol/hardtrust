use chrono::Utc;
use clap::{Parser, Subcommand};
use k256::ecdsa::SigningKey;
use rand_core::OsRng;
use std::os::unix::fs::PermissionsExt;

#[derive(Parser)]
#[command(
    name = "device",
    about = "HardTrust device CLI — identity and data emission"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a real secp256k1 device identity and persist it to ~/.hardtrust/device.key.
    ///
    /// Reads the hardware serial number from the device tree, falling back to
    /// an emulated serial based on the hostname. Prints the serial and Ethereum
    /// address derived from the generated key.
    Init,
    /// Emit a signed sensor reading to reading.json.
    ///
    /// Loads the device key from ~/.hardtrust/device.key, derives the serial
    /// and address, signs the reading, and writes reading.json to the current
    /// directory. Run `device init` first.
    Emit,
}

use device::{create_signed_reading, init_device};

/// Read hardware serial from device tree, or fall back to an emulated serial.
fn read_serial() -> String {
    let hw = std::fs::read_to_string("/sys/firmware/devicetree/base/serial-number")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(serial) = hw {
        return serial;
    }

    let hostname = std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "unknown".to_string());

    format!("EMULATED-{}", hostname)
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let home = std::env::var("HOME").expect("HOME not set");
            let hardtrust_dir = std::path::PathBuf::from(&home).join(".hardtrust");
            let key_path = hardtrust_dir.join("device.key");

            if key_path.exists() {
                println!("Device identity already exists. Run with --force to regenerate.");
                return;
            }

            let serial = read_serial();
            let signing_key = SigningKey::random(&mut OsRng);
            let identity = init_device(&signing_key);

            std::fs::create_dir_all(&hardtrust_dir).expect("failed to create ~/.hardtrust");

            let key_contents = format!("{}\n", identity.key_hex);
            std::fs::write(&key_path, &key_contents).expect("failed to write device.key");
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))
                .expect("failed to set permissions on device.key");

            println!("Serial: {}", serial);
            println!("Address: {}", identity.address);
        }
        Command::Emit => {
            let home = std::env::var("HOME").expect("HOME not set");
            let key_path = std::path::PathBuf::from(&home)
                .join(".hardtrust")
                .join("device.key");

            if !key_path.exists() {
                eprintln!("Device not initialized. Run 'device init' first.");
                std::process::exit(1);
            }

            let key_hex = std::fs::read_to_string(&key_path)
                .expect("failed to read device.key")
                .trim()
                .to_string();
            let key_bytes = hex::decode(&key_hex).expect("invalid key hex in device.key");
            let signing_key =
                SigningKey::from_slice(&key_bytes).expect("invalid key in device.key");

            let serial = read_serial();
            let timestamp = Utc::now().to_rfc3339();
            let reading = create_signed_reading(&signing_key, serial, 22.5, timestamp);

            let json = serde_json::to_string_pretty(&reading).expect("failed to serialize reading");
            std::fs::write("reading.json", &json).expect("failed to write reading.json");
            println!("Wrote reading.json");
        }
    }
}

#[cfg(test)]
mod tests {
    use hardtrust_core::Reading;
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
    fn init_generates_key_and_prints_serial_and_address() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let output = Command::new(device_bin())
            .args(["init"])
            .env("HOME", dir.path())
            .output()
            .expect("failed to run device binary");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Serial:"), "missing Serial line: {stdout}");
        assert!(
            stdout.contains("Address:"),
            "missing Address line: {stdout}"
        );

        let key_path = dir.path().join(".hardtrust").join("device.key");
        assert!(key_path.exists(), "device.key not created");
        let contents = std::fs::read_to_string(&key_path).unwrap();
        let hex = contents.trim();
        assert_eq!(hex.len(), 64, "key should be 32 bytes hex");

        // permissions
        let meta = std::fs::metadata(&key_path).unwrap();
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn init_does_not_overwrite_existing_key() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let hardtrust_dir = dir.path().join(".hardtrust");
        std::fs::create_dir_all(&hardtrust_dir).unwrap();
        let key_path = hardtrust_dir.join("device.key");
        std::fs::write(&key_path, "existingkey\n").unwrap();

        let output = Command::new(device_bin())
            .args(["init"])
            .env("HOME", dir.path())
            .output()
            .expect("failed to run device binary");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.contains("Device identity already exists"),
            "unexpected output: {stdout}"
        );
        // key unchanged
        assert_eq!(std::fs::read_to_string(&key_path).unwrap(), "existingkey\n");
    }

    #[test]
    fn emit_writes_valid_reading_json() {
        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("failed to create work dir");

        // Initialize key first
        Command::new(device_bin())
            .args(["init"])
            .env("HOME", home_dir.path())
            .output()
            .expect("failed to run device init");

        let output = Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device binary");

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Wrote reading.json"), "stdout: {stdout}");

        let contents = std::fs::read_to_string(work_dir.path().join("reading.json"))
            .expect("failed to read file");
        let reading: Reading = serde_json::from_str(&contents).expect("failed to parse JSON");
        assert!(!reading.serial.is_empty(), "serial should not be empty");
        assert!(
            reading.address.starts_with("0x"),
            "address should start with 0x"
        );
        assert!((reading.temperature - 22.5).abs() < f64::EPSILON);
        assert!(
            reading.signature.starts_with("0x") && reading.signature.len() == 132,
            "signature should be 132-char 0x-prefixed hex, got: {}",
            reading.signature
        );
    }

    #[test]
    fn emit_fails_without_key() {
        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("failed to create work dir");

        let output = Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device binary");

        assert!(!output.status.success(), "should exit non-zero without key");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Device not initialized"),
            "stderr: {stderr}"
        );
        assert!(
            !work_dir.path().join("reading.json").exists(),
            "reading.json should not be written"
        );
    }
}
