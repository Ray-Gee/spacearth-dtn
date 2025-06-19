use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryBlock {
    pub version: u8,
    pub destination: String,
    pub source: String,
    pub report_to: String,
    pub creation_timestamp: u64,
    pub lifetime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub primary: PrimaryBlock,
    pub payload: Vec<u8>,
}

impl Bundle {
    pub fn new(source: &str, destination: &str, payload: Vec<u8>) -> Self {
        let creation_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Bundle {
            primary: PrimaryBlock {
                version: 7,
                source: source.to_string(),
                destination: destination.to_string(),
                report_to: "none".to_string(),
                creation_timestamp,
                lifetime: 3600,
            },
            payload,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.primary.creation_timestamp + self.primary.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_primary_block_creation() {
        let primary = PrimaryBlock {
            version: 7,
            destination: "dst://endpoint".to_string(),
            source: "src://endpoint".to_string(),
            report_to: "none".to_string(),
            creation_timestamp: 1234567890,
            lifetime: 3600,
        };

        assert_eq!(primary.version, 7);
        assert_eq!(primary.destination, "dst://endpoint");
        assert_eq!(primary.source, "src://endpoint");
        assert_eq!(primary.report_to, "none");
        assert_eq!(primary.creation_timestamp, 1234567890);
        assert_eq!(primary.lifetime, 3600);
    }

    #[test]
    fn test_bundle_new() {
        let source = "src://test";
        let destination = "dst://test";
        let payload = vec![1, 2, 3, 4];

        let bundle = Bundle::new(source, destination, payload.clone());

        assert_eq!(bundle.primary.version, 7);
        assert_eq!(bundle.primary.source, source);
        assert_eq!(bundle.primary.destination, destination);
        assert_eq!(bundle.primary.report_to, "none");
        assert_eq!(bundle.primary.lifetime, 3600);
        assert_eq!(bundle.payload, payload);

        // Check that timestamp is recent (within last 10 seconds)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(bundle.primary.creation_timestamp <= now);
        assert!(bundle.primary.creation_timestamp > now - 10);
    }

    #[test]
    fn test_bundle_not_expired() {
        let bundle = Bundle::new("src://test", "dst://test", vec![1, 2, 3]);
        assert!(!bundle.is_expired());
    }

    #[test]
    fn test_bundle_expired() {
        let mut bundle = Bundle::new("src://test", "dst://test", vec![1, 2, 3]);

        // Set creation timestamp to 2 hours ago
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        bundle.primary.creation_timestamp = now - 7200; // 2 hours ago

        assert!(bundle.is_expired());
    }

    #[test]
    fn test_bundle_serialization() {
        let bundle = Bundle::new("src://test", "dst://test", vec![1, 2, 3, 4]);

        // Test serialization to JSON
        let json = serde_json::to_string(&bundle).unwrap();
        assert!(json.contains("\"version\":7"));
        assert!(json.contains("\"source\":\"src://test\""));
        assert!(json.contains("\"destination\":\"dst://test\""));

        // Test deserialization from JSON
        let deserialized: Bundle = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.primary.version, bundle.primary.version);
        assert_eq!(deserialized.primary.source, bundle.primary.source);
        assert_eq!(deserialized.primary.destination, bundle.primary.destination);
        assert_eq!(deserialized.payload, bundle.payload);
    }

    #[test]
    fn test_bundle_clone() {
        let original = Bundle::new("src://test", "dst://test", vec![1, 2, 3]);
        let cloned = original.clone();

        assert_eq!(original.primary.version, cloned.primary.version);
        assert_eq!(original.primary.source, cloned.primary.source);
        assert_eq!(original.primary.destination, cloned.primary.destination);
        assert_eq!(
            original.primary.creation_timestamp,
            cloned.primary.creation_timestamp
        );
        assert_eq!(original.payload, cloned.payload);
    }

    #[test]
    fn test_bundle_debug() {
        let bundle = Bundle::new("src://test", "dst://test", vec![1, 2, 3]);
        let debug_str = format!("{:?}", bundle);

        assert!(debug_str.contains("Bundle"));
        assert!(debug_str.contains("PrimaryBlock"));
        assert!(debug_str.contains("src://test"));
        assert!(debug_str.contains("dst://test"));
    }

    #[test]
    fn test_empty_payload() {
        let bundle = Bundle::new("src://test", "dst://test", vec![]);
        assert_eq!(bundle.payload.len(), 0);
        assert!(!bundle.is_expired());
    }

    #[test]
    fn test_large_payload() {
        let large_payload = vec![42u8; 10000];
        let bundle = Bundle::new("src://test", "dst://test", large_payload.clone());
        assert_eq!(bundle.payload.len(), 10000);
        assert_eq!(bundle.payload, large_payload);
    }

    #[test]
    fn test_unicode_endpoints() {
        let source = "src://テスト";
        let destination = "dst://测试";
        let bundle = Bundle::new(source, destination, vec![1, 2, 3]);

        assert_eq!(bundle.primary.source, source);
        assert_eq!(bundle.primary.destination, destination);
    }
}
