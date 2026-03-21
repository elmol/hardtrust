use chrono::Utc;
use clap::{Parser, Subcommand};
use k256::ecdsa::SigningKey;
use rand_core::OsRng;
use std::error::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "device",
    about = "HardTrust device CLI — identity and data emission",
    version
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
    /// Execute a capture command, hash output files, and produce a signed capture.json.
    ///
    /// Runs the given command via `bash -c`, reads all files from the output
    /// directory, computes SHA-256 hashes, signs the manifest, and writes
    /// capture.json to the current directory.
    Capture {
        /// Command to execute for capturing data.
        /// Defaults to /usr/local/lib/terrascope/capture.sh
        #[arg(long, default_value = "/usr/local/lib/terrascope/capture.sh")]
        cmd: String,

        /// Directory where the capture command writes its output files
        #[arg(long, default_value = "./capture-output")]
        output_dir: PathBuf,
    },
}

use device::{
    collect_capture_files, create_signed_capture, create_signed_reading, init_device,
    read_temperature, SYSFS_THERMAL_PATH,
};

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
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
            let hardtrust_dir = std::path::PathBuf::from(&home).join(".hardtrust");
            let key_path = hardtrust_dir.join("device.key");

            if key_path.exists() {
                println!(
                    "Device already initialized. Delete {} to regenerate.",
                    key_path.display()
                );
                return Ok(());
            }

            let serial = read_serial();
            let signing_key = SigningKey::random(&mut OsRng);
            let identity = init_device(&signing_key);

            std::fs::create_dir_all(&hardtrust_dir)
                .map_err(|_| "could not create ~/.hardtrust directory")?;

            let key_contents = format!("{}\n", identity.key_hex);
            std::fs::write(&key_path, &key_contents).map_err(|_| "could not write device key")?;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))
                .map_err(|_| "could not set key file permissions")?;

            println!("Serial: {}", serial);
            println!("Address: {}", identity.address);
        }
        Command::Emit => {
            let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
            let key_path = std::path::PathBuf::from(&home)
                .join(".hardtrust")
                .join("device.key");

            if !key_path.exists() {
                return Err("Device not initialized. Run 'device init' first.".into());
            }

            let key_hex = std::fs::read_to_string(&key_path)
                .map_err(|_| "device.key contains invalid key data")?
                .trim()
                .to_string();
            let key_bytes =
                hex::decode(&key_hex).map_err(|_| "device.key contains invalid key data")?;
            let signing_key = SigningKey::from_slice(&key_bytes)
                .map_err(|_| "device.key contains invalid key data")?;

            let serial = read_serial();
            let timestamp = Utc::now().to_rfc3339();
            let temp_reading = read_temperature(SYSFS_THERMAL_PATH);
            let reading =
                create_signed_reading(&signing_key, serial, temp_reading.celsius, timestamp)?;

            let json = serde_json::to_string_pretty(&reading)
                .map_err(|e| format!("failed to serialize reading: {e}"))?;
            std::fs::write("reading.json", &json).map_err(|_| "failed to write reading.json")?;
            if temp_reading.is_emulated {
                println!("Wrote reading.json [EMULATED temperature]");
            } else {
                println!("Wrote reading.json");
            }
        }
        Command::Capture { cmd, output_dir } => {
            const DEFAULT_CAPTURE_SCRIPT: &str = "/usr/local/lib/terrascope/capture.sh";

            // Check if using default path and it doesn't exist
            if cmd == DEFAULT_CAPTURE_SCRIPT && !std::path::Path::new(&cmd).exists() {
                return Err(format!(
                    "Default capture script not found at {}\n       Install it with: curl -fsSL https://raw.githubusercontent.com/biotexturas/terra-genesis/main/install-device.sh | bash\n       Or specify a custom script: device capture --cmd ./my-script.sh",
                    DEFAULT_CAPTURE_SCRIPT
                ).into());
            }

            let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
            let key_path = std::path::PathBuf::from(&home)
                .join(".hardtrust")
                .join("device.key");

            if !key_path.exists() {
                return Err("Device not initialized. Run 'device init' first.".into());
            }

            let key_hex = std::fs::read_to_string(&key_path)
                .map_err(|_| "device.key contains invalid key data")?
                .trim()
                .to_string();
            let key_bytes =
                hex::decode(&key_hex).map_err(|_| "device.key contains invalid key data")?;
            let signing_key = SigningKey::from_slice(&key_bytes)
                .map_err(|_| "device.key contains invalid key data")?;

            // Create output dir if needed
            std::fs::create_dir_all(&output_dir)
                .map_err(|e| format!("failed to create output dir: {e}"))?;

            // Execute capture command, passing output_dir as argument
            let full_cmd = format!("{} {}", cmd, output_dir.display());
            let output = std::process::Command::new("bash")
                .arg("-c")
                .arg(&full_cmd)
                .output()
                .map_err(|e| format!("failed to execute capture command: {e}"))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "capture command failed (exit {}): {}",
                    output.status.code().unwrap_or(-1),
                    stderr.trim()
                )
                .into());
            }

            // Collect and hash files
            let files = collect_capture_files(&output_dir)
                .map_err(|e| format!("failed to read output dir: {e}"))?;

            if files.is_empty() {
                return Err(format!(
                    "ERROR: No files found in {} after command",
                    output_dir.display()
                )
                .into());
            }

            let serial = read_serial();
            let timestamp = Utc::now().to_rfc3339();
            let capture = create_signed_capture(&signing_key, serial, timestamp, files)?;

            let json = serde_json::to_string_pretty(&capture)
                .map_err(|e| format!("failed to serialize capture: {e}"))?;
            std::fs::write("capture.json", &json).map_err(|_| "failed to write capture.json")?;
            println!("Wrote capture.json ({} file(s))", capture.files.len());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use hardtrust_protocol::{Capture, Reading};
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
            stdout.contains("Device already initialized"),
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
        // Temperature must NOT be the old hardcoded 22.5
        // If real sensor: any valid reading; if emulated: 30.0..=70.0
        assert!(
            (reading.temperature - 22.5).abs() > f64::EPSILON,
            "temperature should not be the old hardcoded 22.5, got: {}",
            reading.temperature
        );
        assert!(
            reading.signature.starts_with("0x") && reading.signature.len() == 132,
            "signature should be 132-char 0x-prefixed hex, got: {}",
            reading.signature
        );
    }

    #[test]
    fn emit_with_corrupted_key_prints_error_not_panic() {
        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("failed to create work dir");
        let hardtrust_dir = home_dir.path().join(".hardtrust");
        std::fs::create_dir_all(&hardtrust_dir).unwrap();
        std::fs::write(hardtrust_dir.join("device.key"), "NOTVALIDHEX\n").unwrap();

        let output = Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn init_with_unwritable_home_prints_error() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        // Create .hardtrust as a FILE so create_dir_all fails
        std::fs::write(dir.path().join(".hardtrust"), "not a directory").unwrap();

        let output = Command::new(device_bin())
            .args(["init"])
            .env("HOME", dir.path())
            .output()
            .expect("failed to run device binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn emit_with_zero_byte_key_prints_error() {
        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("failed to create work dir");
        let hardtrust_dir = home_dir.path().join(".hardtrust");
        std::fs::create_dir_all(&hardtrust_dir).unwrap();
        std::fs::write(hardtrust_dir.join("device.key"), "").unwrap();

        let output = Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn emit_with_truncated_key_prints_error() {
        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("failed to create work dir");
        let hardtrust_dir = home_dir.path().join(".hardtrust");
        std::fs::create_dir_all(&hardtrust_dir).unwrap();
        std::fs::write(hardtrust_dir.join("device.key"), "abcd1234\n").unwrap();

        let output = Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Error:"),
            "expected 'Error:' on stderr, got: {stderr}"
        );
        assert!(!stderr.contains("panic"), "should not panic, got: {stderr}");
    }

    #[test]
    fn emit_prints_emulated_tag() {
        // Validates that emit prints "Wrote reading.json" and, when no real thermal
        // sensor exists, includes "[EMULATED temperature]" in the output.
        let has_real_sensor = std::path::Path::new(device::SYSFS_THERMAL_PATH).exists();

        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("failed to create work dir");

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
            .expect("failed to run device emit");

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.contains("Wrote reading.json"),
            "should print 'Wrote reading.json', got: {stdout}"
        );
        if !has_real_sensor {
            assert!(
                stdout.contains("EMULATED"),
                "should print EMULATED tag without sensor, got: {stdout}"
            );
        }
    }

    #[test]
    fn emit_reading_is_still_verifiable() {
        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("failed to create work dir");

        Command::new(device_bin())
            .args(["init"])
            .env("HOME", home_dir.path())
            .output()
            .expect("failed to run device init");

        Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device emit");

        let contents = std::fs::read_to_string(work_dir.path().join("reading.json"))
            .expect("failed to read reading.json");
        let reading: Reading = serde_json::from_str(&contents).expect("failed to parse JSON");

        // Load device key and verify
        let key_hex = std::fs::read_to_string(home_dir.path().join(".hardtrust/device.key"))
            .unwrap()
            .trim()
            .to_string();
        let key_bytes = hex::decode(&key_hex).unwrap();
        let signing_key = k256::ecdsa::SigningKey::from_slice(&key_bytes).unwrap();
        let address = hardtrust_protocol::public_key_to_address(signing_key.verifying_key());
        assert!(
            hardtrust_protocol::verify_reading(&reading, address),
            "reading signature should verify after temperature source change"
        );
    }

    #[test]
    fn consecutive_emits_produce_different_temperatures() {
        // With emulated random range 30.0..=70.0, two calls are overwhelmingly likely to differ.
        // With a real sensor, readings may be identical — only assert when emulated.
        let has_real_sensor = std::path::Path::new(device::SYSFS_THERMAL_PATH).exists();

        let home_dir = tempfile::tempdir().expect("failed to create temp dir");
        let work_dir1 = tempfile::tempdir().expect("failed to create work dir");
        let work_dir2 = tempfile::tempdir().expect("failed to create work dir");

        Command::new(device_bin())
            .args(["init"])
            .env("HOME", home_dir.path())
            .output()
            .expect("failed to run device init");

        Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir1.path())
            .output()
            .expect("failed to run device emit");

        Command::new(device_bin())
            .args(["emit"])
            .env("HOME", home_dir.path())
            .current_dir(work_dir2.path())
            .output()
            .expect("failed to run device emit");

        let r1: Reading = serde_json::from_str(
            &std::fs::read_to_string(work_dir1.path().join("reading.json")).unwrap(),
        )
        .unwrap();
        let r2: Reading = serde_json::from_str(
            &std::fs::read_to_string(work_dir2.path().join("reading.json")).unwrap(),
        )
        .unwrap();

        if has_real_sensor {
            // Both should be real sensor readings (not 22.5)
            assert!(
                (r1.temperature - 22.5).abs() > f64::EPSILON,
                "temperature should not be hardcoded 22.5"
            );
        } else {
            assert!(
                (r1.temperature - r2.temperature).abs() > f64::EPSILON,
                "consecutive emulated emits should produce different temperatures: {} vs {}",
                r1.temperature,
                r2.temperature
            );
        }
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

    // --- Capture integration tests ---

    fn init_device_key(home_dir: &std::path::Path) {
        Command::new(device_bin())
            .args(["init"])
            .env("HOME", home_dir)
            .output()
            .expect("failed to run device init");
    }

    #[test]
    fn capture_single_file_produces_valid_capture_json() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let output_dir = work_dir.path().join("output");
        init_device_key(home_dir.path());

        let cmd = format!("echo hello > {}/test.txt", output_dir.display());
        let output = Command::new(device_bin())
            .args(["capture", "--cmd", &cmd, "--output-dir"])
            .arg(&output_dir)
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            output.status.success(),
            "capture failed: stdout={stdout} stderr={stderr}"
        );
        assert!(stdout.contains("Wrote capture.json"), "stdout: {stdout}");

        let contents = std::fs::read_to_string(work_dir.path().join("capture.json")).unwrap();
        let capture: Capture = serde_json::from_str(&contents).unwrap();
        assert_eq!(capture.files.len(), 1);
        assert_eq!(capture.files[0].name, "test.txt");
        assert!(capture.files[0].hash.starts_with("sha256:"));
        assert!(capture.signature.starts_with("0x") && capture.signature.len() == 132);
    }

    #[test]
    fn capture_multiple_files_deterministic_content_hash() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let output_dir = work_dir.path().join("output");
        init_device_key(home_dir.path());

        let cmd = format!(
            "echo alpha > {0}/a.txt && echo bravo > {0}/b.txt",
            output_dir.display()
        );
        let output = Command::new(device_bin())
            .args(["capture", "--cmd", &cmd, "--output-dir"])
            .arg(&output_dir)
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        assert!(output.status.success());
        let contents = std::fs::read_to_string(work_dir.path().join("capture.json")).unwrap();
        let capture: Capture = serde_json::from_str(&contents).unwrap();
        assert_eq!(capture.files.len(), 2);
        // Files sorted alphabetically
        assert_eq!(capture.files[0].name, "a.txt");
        assert_eq!(capture.files[1].name, "b.txt");
        assert!(capture.content_hash.starts_with("sha256:"));
    }

    #[test]
    fn capture_command_failure_aborts_with_error() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let output_dir = work_dir.path().join("output");
        init_device_key(home_dir.path());

        let output = Command::new(device_bin())
            .args(["capture", "--cmd", "exit 1", "--output-dir"])
            .arg(&output_dir)
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Error:"), "stderr: {stderr}");
        assert!(
            !work_dir.path().join("capture.json").exists(),
            "capture.json should not be written"
        );
    }

    #[test]
    fn capture_empty_output_aborts_with_error() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let output_dir = work_dir.path().join("output");
        init_device_key(home_dir.path());

        // Command succeeds but produces no files
        let output = Command::new(device_bin())
            .args(["capture", "--cmd", "true", "--output-dir"])
            .arg(&output_dir)
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("No files found"),
            "expected 'No files found' error, got: {stderr}"
        );
    }

    #[test]
    fn capture_signature_verifies_with_protocol() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let output_dir = work_dir.path().join("output");
        init_device_key(home_dir.path());

        let cmd = format!("echo test > {}/file.txt", output_dir.display());
        Command::new(device_bin())
            .args(["capture", "--cmd", &cmd, "--output-dir"])
            .arg(&output_dir)
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        let contents = std::fs::read_to_string(work_dir.path().join("capture.json")).unwrap();
        let capture: Capture = serde_json::from_str(&contents).unwrap();

        // Load key and verify
        let key_hex = std::fs::read_to_string(home_dir.path().join(".hardtrust/device.key"))
            .unwrap()
            .trim()
            .to_string();
        let key_bytes = hex::decode(&key_hex).unwrap();
        let signing_key = k256::ecdsa::SigningKey::from_slice(&key_bytes).unwrap();
        let address = hardtrust_protocol::public_key_to_address(signing_key.verifying_key());
        assert!(
            hardtrust_protocol::verify(&capture, address),
            "capture signature should verify"
        );
    }

    #[test]
    fn capture_file_manifest_matches_output() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let output_dir = work_dir.path().join("output");
        init_device_key(home_dir.path());

        let cmd = format!(
            "echo one > {0}/x.txt && echo two > {0}/y.txt",
            output_dir.display()
        );
        Command::new(device_bin())
            .args(["capture", "--cmd", &cmd, "--output-dir"])
            .arg(&output_dir)
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        let contents = std::fs::read_to_string(work_dir.path().join("capture.json")).unwrap();
        let capture: Capture = serde_json::from_str(&contents).unwrap();

        // Verify manifest matches actual files
        let mut actual_files: Vec<String> = std::fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(|e| {
                let e = e.ok()?;
                if e.file_type().ok()?.is_file() {
                    Some(e.file_name().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();
        actual_files.sort();

        let manifest_names: Vec<String> = capture.files.iter().map(|f| f.name.clone()).collect();
        assert_eq!(manifest_names, actual_files);
    }

    // --- Default --cmd tests ---

    #[test]
    fn capture_without_cmd_uses_default_path_and_shows_error_when_missing() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        init_device_key(home_dir.path());

        // Run without --cmd, default script won't exist
        let output = Command::new(device_bin())
            .args(["capture", "--output-dir"])
            .arg(work_dir.path().join("output"))
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("Default capture script not found"),
            "expected helpful error about missing default script, got: {stderr}"
        );
        assert!(
            stderr.contains("/usr/local/lib/terrascope/capture.sh"),
            "should mention the default path, got: {stderr}"
        );
        assert!(
            stderr.contains("--cmd"),
            "should suggest --cmd override, got: {stderr}"
        );
    }

    #[test]
    fn capture_with_explicit_cmd_still_works() {
        let home_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let output_dir = work_dir.path().join("output");
        init_device_key(home_dir.path());

        let cmd = format!("echo override > {}/test.txt", output_dir.display());
        let output = Command::new(device_bin())
            .args(["capture", "--cmd", &cmd, "--output-dir"])
            .arg(&output_dir)
            .env("HOME", home_dir.path())
            .current_dir(work_dir.path())
            .output()
            .expect("failed to run device capture");

        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            output.status.success(),
            "explicit --cmd should work: stdout={stdout} stderr={stderr}"
        );
        assert!(stdout.contains("Wrote capture.json"), "stdout: {stdout}");
    }
}
