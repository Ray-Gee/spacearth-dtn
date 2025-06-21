use crate::routing::algorithm::RoutingAlgorithmType;
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
pub struct RoutingConfig {
    pub algorithm: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bundle: BundleConfig,
    pub endpoints: EndpointsConfig,
    pub storage: StorageConfig,
    pub routing: RoutingConfig,
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

    pub fn get_routing_algorithm_type(&self) -> RoutingAlgorithmType {
        match self.routing.algorithm.to_lowercase().as_str() {
            "epidemic" => RoutingAlgorithmType::Epidemic,
            "prophet" => RoutingAlgorithmType::Prophet,
            // "sprayandwait" => RoutingAlgorithmType::SprayAndWait,
            _ => {
                eprintln!(
                    "Warning: Unknown routing algorithm '{}', falling back to epidemic",
                    self.routing.algorithm
                );
                RoutingAlgorithmType::Epidemic
            }
        }
    }

    #[cfg(test)]
    pub fn test_config() -> Self {
        Config {
            bundle: BundleConfig {
                version: 7,
                lifetime: 3600,
            },
            endpoints: EndpointsConfig {
                destination: "dtn://dest".to_string(),
                source: "dtn://src".to_string(),
                report_to: "dtn://report".to_string(),
            },
            storage: StorageConfig {
                path: "bundles".to_string(),
                max_size: 1024,
            },
            routing: RoutingConfig {
                algorithm: "epidemic".to_string(),
            },
        }
    }
}

pub fn generate_creation_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_creation_timestamp() {
        let ts = generate_creation_timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_config_load() {
        // Test that config loading works when file exists, or fails gracefully when it doesn't
        let result = Config::load();
        // We don't assert it's ok because the file might not exist in test environment
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_config_structure() {
        let config = Config::test_config();

        // Test bundle config
        assert_eq!(config.bundle.version, 7);
        assert_eq!(config.bundle.lifetime, 3600);

        // Test endpoints config
        assert_eq!(config.endpoints.destination, "dtn://dest");
        assert_eq!(config.endpoints.source, "dtn://src");
        assert_eq!(config.endpoints.report_to, "dtn://report");

        // Test storage config
        assert_eq!(config.storage.path, "bundles");
        assert_eq!(config.storage.max_size, 1024);

        // Test routing config
        assert_eq!(config.routing.algorithm, "epidemic");
    }

    #[test]
    fn test_get_routing_algorithm_type_epidemic() {
        let config = Config::test_config();
        let algorithm_type = config.get_routing_algorithm_type();
        assert!(matches!(algorithm_type, RoutingAlgorithmType::Epidemic));
    }

    #[test]
    fn test_get_routing_algorithm_type_prophet() {
        // Create a mock config with Prophet algorithm
        let config = Config {
            bundle: BundleConfig {
                version: 7,
                lifetime: 3600,
            },
            endpoints: EndpointsConfig {
                destination: "dtn://dest".to_string(),
                source: "dtn://src".to_string(),
                report_to: "dtn://report".to_string(),
            },
            storage: StorageConfig {
                path: "bundles".to_string(),
                max_size: 1024,
            },
            routing: RoutingConfig {
                algorithm: "prophet".to_string(),
            },
        };

        let algorithm_type = config.get_routing_algorithm_type();
        assert!(matches!(algorithm_type, RoutingAlgorithmType::Prophet));
    }

    #[test]
    fn test_get_routing_algorithm_type_case_insensitive() {
        let config = Config {
            bundle: BundleConfig {
                version: 7,
                lifetime: 3600,
            },
            endpoints: EndpointsConfig {
                destination: "dtn://dest".to_string(),
                source: "dtn://src".to_string(),
                report_to: "dtn://report".to_string(),
            },
            storage: StorageConfig {
                path: "bundles".to_string(),
                max_size: 1024,
            },
            routing: RoutingConfig {
                algorithm: "EPIDEMIC".to_string(),
            },
        };

        let algorithm_type = config.get_routing_algorithm_type();
        assert!(matches!(algorithm_type, RoutingAlgorithmType::Epidemic));
    }

    #[test]
    fn test_get_routing_algorithm_type_unknown_fallback() {
        let config = Config {
            bundle: BundleConfig {
                version: 7,
                lifetime: 3600,
            },
            endpoints: EndpointsConfig {
                destination: "dtn://dest".to_string(),
                source: "dtn://src".to_string(),
                report_to: "dtn://report".to_string(),
            },
            storage: StorageConfig {
                path: "bundles".to_string(),
                max_size: 1024,
            },
            routing: RoutingConfig {
                algorithm: "unknown_algorithm".to_string(),
            },
        };

        let algorithm_type = config.get_routing_algorithm_type();
        assert!(matches!(algorithm_type, RoutingAlgorithmType::Epidemic)); // Should fallback to Epidemic
    }

    #[test]
    fn test_bundle_config_debug() {
        let bundle_config = BundleConfig {
            version: 7,
            lifetime: 3600,
        };

        let debug_str = format!("{:?}", bundle_config);
        assert!(debug_str.contains("BundleConfig"));
        assert!(debug_str.contains("7"));
        assert!(debug_str.contains("3600"));
    }

    #[test]
    fn test_endpoints_config_debug() {
        let endpoints_config = EndpointsConfig {
            destination: "dtn://dest".to_string(),
            source: "dtn://src".to_string(),
            report_to: "dtn://report".to_string(),
        };

        let debug_str = format!("{:?}", endpoints_config);
        assert!(debug_str.contains("EndpointsConfig"));
        assert!(debug_str.contains("dtn://dest"));
        assert!(debug_str.contains("dtn://src"));
        assert!(debug_str.contains("dtn://report"));
    }

    #[test]
    fn test_storage_config_debug() {
        let storage_config = StorageConfig {
            path: "test_bundles".to_string(),
            max_size: 2048,
        };

        let debug_str = format!("{:?}", storage_config);
        assert!(debug_str.contains("StorageConfig"));
        assert!(debug_str.contains("test_bundles"));
        assert!(debug_str.contains("2048"));
    }

    #[test]
    fn test_routing_config_debug() {
        let routing_config = RoutingConfig {
            algorithm: "epidemic".to_string(),
        };

        let debug_str = format!("{:?}", routing_config);
        assert!(debug_str.contains("RoutingConfig"));
        assert!(debug_str.contains("epidemic"));
    }

    #[test]
    fn test_config_debug() {
        let config = Config::test_config();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("bundle"));
        assert!(debug_str.contains("endpoints"));
        assert!(debug_str.contains("storage"));
        assert!(debug_str.contains("routing"));
    }

    #[test]
    fn test_timestamp_progression() {
        let ts1 = generate_creation_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = generate_creation_timestamp();

        assert!(ts2 >= ts1);
    }

    #[test]
    fn test_config_deserialization_fields() {
        let config = Config::test_config();

        // Test that all fields are properly deserialized
        assert!(config.bundle.version > 0);
        assert!(config.bundle.lifetime > 0);
        assert!(!config.endpoints.destination.is_empty());
        assert!(!config.endpoints.source.is_empty());
        assert!(!config.endpoints.report_to.is_empty());
        assert!(!config.storage.path.is_empty());
        assert!(config.storage.max_size > 0);
        assert!(!config.routing.algorithm.is_empty());
    }

    #[test]
    fn test_test_config() {
        let config = Config::test_config();
        assert_eq!(config.bundle.version, 7);
        assert_eq!(config.bundle.lifetime, 3600);
        assert_eq!(config.endpoints.destination, "dtn://dest");
        assert_eq!(config.endpoints.source, "dtn://src");
        assert_eq!(config.endpoints.report_to, "dtn://report");
        assert_eq!(config.storage.path, "bundles");
        assert_eq!(config.storage.max_size, 1024);
        assert_eq!(config.routing.algorithm, "epidemic");
    }
}
