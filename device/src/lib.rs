use hardtrust_protocol::{
    public_key_to_address, sign, sign_reading, Capture, CaptureFile, ProtocolError, Reading,
};
use k256::ecdsa::SigningKey;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Default sysfs thermal sensor path.
pub const SYSFS_THERMAL_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";

/// Result of reading a temperature sensor.
pub struct TemperatureReading {
    /// Temperature in degrees Celsius.
    pub celsius: f64,
    /// Whether the value was emulated (sensor unavailable).
    pub is_emulated: bool,
}

/// Read CPU temperature from sysfs, or return a simulated value if unavailable.
///
/// Takes the sensor file path as a parameter for testability.
pub fn read_temperature(sensor_path: &str) -> TemperatureReading {
    if let Ok(contents) = fs::read_to_string(sensor_path) {
        if let Ok(millidegrees) = contents.trim().parse::<i64>() {
            return TemperatureReading {
                celsius: millidegrees as f64 / 1000.0,
                is_emulated: false,
            };
        }
    }
    let mut rng = rand::thread_rng();
    let value: f64 = rng.gen_range(30.0..=70.0);
    TemperatureReading {
        celsius: value,
        is_emulated: true,
    }
}

/// Identity derived from a device's signing key.
pub struct DeviceIdentity {
    /// Ethereum address derived from the public key.
    pub address: String,
    /// 64-character lowercase hex of the private key bytes.
    pub key_hex: String,
}

/// Derive a `DeviceIdentity` from an already-generated signing key.
///
/// Pure: no file I/O, no env vars, no randomness.
pub fn init_device(signing_key: &SigningKey) -> DeviceIdentity {
    let address = public_key_to_address(signing_key.verifying_key());
    let key_hex = hex::encode(signing_key.to_bytes());
    DeviceIdentity {
        address: format!("{}", address),
        key_hex,
    }
}

/// Construct and sign a `Reading` from the given key and parameters.
///
/// Pure: temperature and timestamp are explicit parameters, not read from
/// clock or sensors. No file I/O.
/// Returns `Err` only if signing fails due to invalid address or timestamp.
pub fn create_signed_reading(
    signing_key: &SigningKey,
    serial: String,
    temperature: f64,
    timestamp: String,
) -> Result<Reading, ProtocolError> {
    let address = public_key_to_address(signing_key.verifying_key());
    let mut reading = Reading {
        serial,
        address: format!("{}", address),
        temperature,
        timestamp,
        signature: String::new(),
    };
    reading.signature = sign_reading(signing_key, &reading)?;
    Ok(reading)
}

/// Compute SHA-256 hash of a file's contents.
pub fn hash_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let hash = Sha256::digest(&data);
    Ok(format!("sha256:{}", hex::encode(hash)))
}

/// Compute the content hash from a sorted list of CaptureFiles.
///
/// Deterministic: hashes (name + individual_hash) for all files in order.
pub fn compute_content_hash(files: &[CaptureFile]) -> String {
    let mut hasher = Sha256::new();
    for f in files {
        hasher.update(f.name.as_bytes());
        hasher.update(f.hash.strip_prefix("sha256:").unwrap().as_bytes());
    }
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Read all files in a directory, compute hashes, return sorted CaptureFiles.
///
/// Non-recursive: skips subdirectories.
pub fn collect_capture_files(
    output_dir: &Path,
) -> Result<Vec<CaptureFile>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let hash = hash_file(&entry.path())?;
        let size = entry.metadata()?.len();
        files.push(CaptureFile { name, hash, size });
    }
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

/// Construct and sign a `Capture` from the given key and captured files.
///
/// Pure: timestamp is an explicit parameter. No file I/O.
pub fn create_signed_capture(
    signing_key: &SigningKey,
    serial: String,
    timestamp: String,
    files: Vec<CaptureFile>,
) -> Result<Capture, ProtocolError> {
    let address = public_key_to_address(signing_key.verifying_key());
    let content_hash = compute_content_hash(&files);
    let mut capture = Capture {
        serial,
        address: format!("{}", address),
        timestamp,
        content_hash,
        files,
        signature: String::new(),
    };
    capture.signature = sign(signing_key, &capture)?;
    Ok(capture)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hardtrust_protocol::{public_key_to_address, verify, verify_reading};
    use std::io::Write;

    fn test_signing_key() -> SigningKey {
        let key_bytes =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("valid hex");
        SigningKey::from_slice(&key_bytes).expect("valid key")
    }

    #[test]
    fn init_device_produces_correct_address() {
        let key = test_signing_key();
        let expected = public_key_to_address(key.verifying_key());
        let identity = init_device(&key);
        assert_eq!(identity.address, format!("{}", expected));
    }

    #[test]
    fn init_device_key_hex_round_trips() {
        let key = test_signing_key();
        let identity = init_device(&key);
        assert_eq!(identity.key_hex.len(), 64);
        let bytes = hex::decode(&identity.key_hex).expect("valid hex");
        let recovered = SigningKey::from_slice(&bytes).expect("valid key");
        let addr_original = public_key_to_address(key.verifying_key());
        let addr_recovered = public_key_to_address(recovered.verifying_key());
        assert_eq!(addr_original, addr_recovered);
    }

    #[test]
    fn create_signed_reading_produces_valid_signature() {
        let key = test_signing_key();
        let address = public_key_to_address(key.verifying_key());
        let reading = create_signed_reading(
            &key,
            "TEST-001".to_string(),
            22.5,
            "2026-01-01T00:00:00Z".to_string(),
        )
        .expect("valid reading");
        assert!(verify_reading(&reading, address), "signature should verify");
    }

    #[test]
    fn read_temperature_valid_sysfs() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "42500\n").unwrap();
        let r = read_temperature(tmp.path().to_str().unwrap());
        assert!((r.celsius - 42.5).abs() < f64::EPSILON);
        assert!(!r.is_emulated);
    }

    #[test]
    fn read_temperature_trailing_whitespace() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "55000\n  ").unwrap();
        let r = read_temperature(tmp.path().to_str().unwrap());
        assert!((r.celsius - 55.0).abs() < f64::EPSILON);
        assert!(!r.is_emulated);
    }

    #[test]
    fn read_temperature_fallback_missing_file() {
        let r = read_temperature("/tmp/nonexistent-hardtrust-sensor-test");
        assert!(r.is_emulated);
        assert!(r.celsius >= 30.0 && r.celsius <= 70.0);
    }

    #[test]
    fn read_temperature_fallback_non_numeric() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "not-a-number\n").unwrap();
        let r = read_temperature(tmp.path().to_str().unwrap());
        assert!(r.is_emulated);
        assert!(r.celsius >= 30.0 && r.celsius <= 70.0);
    }

    #[test]
    fn read_temperature_fallback_empty_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let r = read_temperature(tmp.path().to_str().unwrap());
        assert!(r.is_emulated);
        assert!(r.celsius >= 30.0 && r.celsius <= 70.0);
    }

    #[test]
    fn read_temperature_emulated_values_vary() {
        let values: Vec<f64> = (0..10)
            .map(|_| read_temperature("/tmp/nonexistent-hardtrust-sensor-test").celsius)
            .collect();
        let distinct = values
            .windows(2)
            .any(|w| (w[0] - w[1]).abs() > f64::EPSILON);
        assert!(distinct, "emulated values should vary, got: {:?}", values);
    }

    #[test]
    fn create_signed_reading_is_deterministic() {
        let key = test_signing_key();
        let r1 = create_signed_reading(
            &key,
            "TEST-001".to_string(),
            22.5,
            "2026-01-01T00:00:00Z".to_string(),
        )
        .expect("valid reading");
        let r2 = create_signed_reading(
            &key,
            "TEST-001".to_string(),
            22.5,
            "2026-01-01T00:00:00Z".to_string(),
        )
        .expect("valid reading");
        assert_eq!(r1.signature, r2.signature);
        assert_eq!(r1.temperature, r2.temperature);
        assert_eq!(r1.timestamp, r2.timestamp);
    }

    // --- Capture lib tests ---

    #[test]
    fn hash_file_computes_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello").unwrap();
        let hash = hash_file(&file_path).unwrap();
        assert!(hash.starts_with("sha256:"));
        // SHA-256 of "hello"
        assert_eq!(
            hash,
            "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn compute_content_hash_is_deterministic() {
        let files = vec![
            CaptureFile {
                name: "a.txt".to_string(),
                hash: "sha256:aaaa".to_string(),
                size: 10,
            },
            CaptureFile {
                name: "b.txt".to_string(),
                hash: "sha256:bbbb".to_string(),
                size: 20,
            },
        ];
        let h1 = compute_content_hash(&files);
        let h2 = compute_content_hash(&files);
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
    }

    #[test]
    fn collect_capture_files_reads_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("b.txt"), "bravo").unwrap();
        std::fs::write(dir.path().join("a.txt"), "alpha").unwrap();
        let files = collect_capture_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        // Should be sorted alphabetically
        assert_eq!(files[0].name, "a.txt");
        assert_eq!(files[1].name, "b.txt");
        assert!(files[0].hash.starts_with("sha256:"));
        assert_eq!(files[0].size, 5); // "alpha" = 5 bytes
        assert_eq!(files[1].size, 5); // "bravo" = 5 bytes
    }

    #[test]
    fn collect_capture_files_skips_subdirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file.txt"), "data").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        let files = collect_capture_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn create_signed_capture_produces_valid_signature() {
        let key = test_signing_key();
        let address = public_key_to_address(key.verifying_key());
        let files = vec![CaptureFile {
            name: "test.txt".to_string(),
            hash: "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
                .to_string(),
            size: 5,
        }];
        let capture = create_signed_capture(
            &key,
            "TEST-001".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            files,
        )
        .expect("valid capture");
        assert!(verify(&capture, address));
    }

    #[test]
    fn create_signed_capture_content_hash_matches_files() {
        let key = test_signing_key();
        let files = vec![CaptureFile {
            name: "test.txt".to_string(),
            hash: "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
                .to_string(),
            size: 5,
        }];
        let expected_content_hash = compute_content_hash(&files);
        let capture = create_signed_capture(
            &key,
            "TEST-001".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            files,
        )
        .expect("valid capture");
        assert_eq!(capture.content_hash, expected_content_hash);
    }
}
