use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct BundleConfig {
    pub version: u8,
    pub lifetime: u64,
}

#[derive(Debug, Deserialize)]
pub struct EndpointsConfig {
    pub destination: String,
    pub source: String,
    pub report_to: String,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub path: String,
    pub max_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bundle: BundleConfig,
    pub endpoints: EndpointsConfig,
    pub storage: StorageConfig,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_path =
            std::env::var("DTN_CONFIG").unwrap_or_else(|_| "config/default.toml".to_string());

        let settings = config::Config::builder()
            .add_source(config::File::from(Path::new(&config_path)))
            .add_source(config::Environment::with_prefix("DTN"))
            .build()?;

        settings.try_deserialize()
    }
}

pub fn generate_creation_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
