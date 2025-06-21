use crate::bpv7::bundle::Bundle;

/// Bundle status information
#[derive(Debug)]
pub enum BundleStatus {
    /// Status of a single bundle
    Single { id: String, bundle: Bundle },
    /// Summary status of all bundles
    Summary {
        active: usize,
        expired: usize,
        total: usize,
    },
}
