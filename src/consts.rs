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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_VERSION, 7);
        assert_eq!(DEFAULT_LIFETIME, 3600);
        assert_eq!(DEFAULT_REPORT_TO, "none");
        assert_eq!(BUNDLES_DIR, "./bundles");
        assert_eq!(DISPATCHED_DIR, "./dispatched");
    }

    #[test]
    fn test_ble_constants() {
        assert_eq!(ble::SERVICE_UUID, "12345678-1234-5678-1234-56789abcdef0");
        assert_eq!(ble::WRITE_CHAR_UUID, "12345678-1234-5678-1234-56789abcdef1");
        assert_eq!(
            ble::NOTIFY_CHAR_UUID,
            "12345678-1234-5678-1234-56789abcdef2"
        );
        assert_eq!(ble::ADV_NAME, "spacearth-dtn-ble");
        assert_eq!(ble::ACK, b"ACK\n");
    }

    #[test]
    fn test_tcp_constants() {
        assert_eq!(tcp::ACK, "ACK");
        assert_eq!(tcp::OK, "OK");
        assert_eq!(tcp::SUCCESS, "SUCCESS");
        assert_eq!(tcp::RECEIVED, "RECEIVED");
    }

    #[test]
    fn test_ble_uuid_format() {
        let service_uuid = ble::SERVICE_UUID;

        // Check UUID format (8-4-4-4-12)
        let parts: Vec<&str> = service_uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);

        // Check that UUIDs are different
        assert_ne!(ble::SERVICE_UUID, ble::WRITE_CHAR_UUID);
        assert_ne!(ble::SERVICE_UUID, ble::NOTIFY_CHAR_UUID);
        assert_ne!(ble::WRITE_CHAR_UUID, ble::NOTIFY_CHAR_UUID);
    }

    #[test]
    fn test_ack_bytes() {
        let ack_bytes = ble::ACK;
        assert_eq!(ack_bytes.len(), 4);
        assert_eq!(ack_bytes[0], b'A');
        assert_eq!(ack_bytes[1], b'C');
        assert_eq!(ack_bytes[2], b'K');
        assert_eq!(ack_bytes[3], b'\n');
    }

    #[test]
    fn test_directory_paths() {
        assert!(BUNDLES_DIR.starts_with("./"));
        assert!(DISPATCHED_DIR.starts_with("./"));
        assert_ne!(BUNDLES_DIR, DISPATCHED_DIR);
    }

    #[test]
    fn test_tcp_string_constants_not_empty() {
        assert!(!tcp::ACK.is_empty());
        assert!(!tcp::OK.is_empty());
        assert!(!tcp::SUCCESS.is_empty());
        assert!(!tcp::RECEIVED.is_empty());

        // Check they are different
        assert_ne!(tcp::ACK, tcp::OK);
        assert_ne!(tcp::ACK, tcp::SUCCESS);
        assert_ne!(tcp::ACK, tcp::RECEIVED);
    }

    #[test]
    fn test_lifetime_is_positive() {
        assert!(DEFAULT_LIFETIME > 0);
        assert_eq!(DEFAULT_LIFETIME, 60 * 60); // 1 hour in seconds
    }
}
