use crate::bpv7::EndpointId;
use async_trait::async_trait;

/// Common interface for all CLA peer types
/// Provides abstraction over different CLA implementations (TCP, BLE, etc.)
/// for use in routing algorithms and CLA management
#[async_trait]
pub trait ClaPeer: Send + Sync {
    /// Get the endpoint ID of this peer
    fn get_peer_endpoint_id(&self) -> EndpointId;

    /// Check if this peer is currently reachable/connectable
    /// This method should be implemented by each CLA type to perform
    /// appropriate connectivity checks (TCP ping, BLE scan, etc.)
    async fn is_reachable(&self) -> bool;

    /// Get the CLA type for this peer (e.g., "tcp", "ble", "udp")
    fn get_cla_type(&self) -> &str;

    /// Get the connection address/identifier for this peer
    fn get_connection_address(&self) -> String;

    /// Clone this peer into a boxed trait object
    fn clone_box(&self) -> Box<dyn ClaPeer>;

    /// Activate this peer's convergence layer
    /// This delegates to the underlying ConvergenceLayer implementation
    async fn activate(&self) -> anyhow::Result<()>;
}

/// Enable cloning for boxed ClaPeer trait objects
impl Clone for Box<dyn ClaPeer> {
    fn clone(&self) -> Box<dyn ClaPeer> {
        self.clone_box()
    }
}
