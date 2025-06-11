use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimaryBlock {
    pub version: u8,
    pub destination: String,
    pub source: String,
    pub report_to: String,
    pub creation_timestamp: u64,
    pub lifetime: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub primary: PrimaryBlock,
    pub payload: Vec<u8>,
}
