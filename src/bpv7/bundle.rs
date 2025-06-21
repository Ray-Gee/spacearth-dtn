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
