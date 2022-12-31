use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_unix_epoch_millis() -> u128 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .expect("Time went backward")
        .as_millis()
}

pub fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        write!(&mut out, "{:02X}", byte).expect("generating HEX fails");
    }
    out
}
