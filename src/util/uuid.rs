use std::time::SystemTime;

use chacha20poly1305::aead::OsRng;
use rand::RngCore;

/// generates a UUID 7 string using a unix timestamp and random bytes.
pub fn generate_uuid() -> String {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let mut random_bytes = [0u8; 10];
    OsRng.fill_bytes(&mut random_bytes);

    uuid::Builder::from_unix_timestamp_millis(ts.as_millis().try_into().unwrap(), &random_bytes)
        .into_uuid()
        .to_string()
}
