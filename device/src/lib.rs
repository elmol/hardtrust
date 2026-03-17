use hardtrust_protocol::{public_key_to_address, sign_reading, ProtocolError, Reading};
use k256::ecdsa::SigningKey;
use rand::Rng;
use std::fs;

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

#[cfg(test)]
mod tests {
    use super::*;
    use hardtrust_protocol::{public_key_to_address, verify_reading};
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
}
