use serde::{Deserialize, Serialize};
use std::fmt;

/// Endpoint Identifier (EID) as defined in BPv7 specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EndpointId(String);

impl EndpointId {
    /// Create a new EndpointId
    pub fn new(id: String) -> Self {
        EndpointId(id)
    }

    /// Create EndpointId from string slice
    pub fn from(id: &str) -> Self {
        EndpointId(id.to_string())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a valid DTN scheme EID
    pub fn is_dtn_scheme(&self) -> bool {
        self.0.starts_with("dtn://")
    }

    /// Check if this is a null endpoint
    pub fn is_null(&self) -> bool {
        self.0 == "dtn:none" || self.0.is_empty()
    }
}

impl From<String> for EndpointId {
    fn from(id: String) -> Self {
        EndpointId(id)
    }
}

impl From<&str> for EndpointId {
    fn from(id: &str) -> Self {
        EndpointId(id.to_string())
    }
}

impl fmt::Display for EndpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
