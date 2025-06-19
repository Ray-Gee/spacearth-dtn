pub const DEFAULT_VERSION: u8 = 7;
pub const DEFAULT_LIFETIME: u64 = 3600;
pub const DEFAULT_REPORT_TO: &str = "none";
pub const BUNDLES_DIR: &str = "./bundles";
pub const DISPATCHED_DIR: &str = "./dispatched";

pub mod ble {
    pub const SERVICE_UUID: &str = "12345678-1234-5678-1234-56789abcdef0";
    pub const WRITE_CHAR_UUID: &str = "12345678-1234-5678-1234-56789abcdef1";
    pub const NOTIFY_CHAR_UUID: &str = "12345678-1234-5678-1234-56789abcdef2";
    pub const ADV_NAME: &str = "spacearth-dtn-ble";
    pub const ACK: &[u8] = b"ACK\n";
}

pub mod tcp {
    pub const ACK: &str = "ACK";
    pub const OK: &str = "OK";
    pub const SUCCESS: &str = "SUCCESS";
    pub const RECEIVED: &str = "RECEIVED";
}
