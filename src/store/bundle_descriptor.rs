use crate::bpv7::bundle::Bundle;
use crate::bpv7::endpoint::EndpointId;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// BundleDescriptor manages the forwarding state of a bundle
/// It tracks which endpoints have already received this bundle to prevent duplicates
#[derive(Debug, Clone)]
pub struct BundleDescriptor {
    pub bundle: Bundle,
    pub already_sent: HashSet<EndpointId>,
    pub forwarding_attempts: u32,
    pub created_at: u64,
}

impl BundleDescriptor {
    pub fn new(bundle: Bundle) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            bundle,
            already_sent: HashSet::new(),
            forwarding_attempts: 0,
            created_at: now,
        }
    }

    pub fn mark_sent(&mut self, eid: EndpointId) {
        self.already_sent.insert(eid);
    }

    pub fn get_already_sent(&self) -> &HashSet<EndpointId> {
        &self.already_sent
    }

    /// Check if this bundle has been sent to a specific endpoint
    pub fn has_been_sent_to(&self, eid: &EndpointId) -> bool {
        self.already_sent.contains(eid)
    }

    /// Increment the forwarding attempt counter
    pub fn increment_forwarding_attempts(&mut self) {
        self.forwarding_attempts += 1;
    }

    /// Get the number of forwarding attempts
    pub fn get_forwarding_attempts(&self) -> u32 {
        self.forwarding_attempts
    }

    /// Check if this bundle is ready for forwarding (not expired and not over limit)
    pub fn is_ready_for_forwarding(&self, max_attempts: u32) -> bool {
        !self.bundle.is_expired() && self.forwarding_attempts < max_attempts
    }

    /// Get a unique identifier for this bundle
    pub fn get_bundle_id(&self) -> String {
        format!(
            "{}-{}",
            self.bundle.primary.source, self.bundle.primary.creation_timestamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpv7::bundle::Bundle;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_bundle_descriptor_new() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let descriptor = BundleDescriptor::new(bundle.clone());

        assert_eq!(descriptor.bundle.primary.source, bundle.primary.source);
        assert_eq!(
            descriptor.bundle.primary.destination,
            bundle.primary.destination
        );
        assert_eq!(descriptor.bundle.payload, bundle.payload);
        assert_eq!(descriptor.forwarding_attempts, 0);
        assert!(descriptor.already_sent.is_empty());
        assert!(descriptor.created_at > 0);
    }

    #[test]
    fn test_mark_sent() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        let eid = EndpointId::from("dtn://peer1");
        descriptor.mark_sent(eid.clone());

        assert!(descriptor.already_sent.contains(&eid));
        assert_eq!(descriptor.already_sent.len(), 1);
    }

    #[test]
    fn test_mark_sent_multiple() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        let eid1 = EndpointId::from("dtn://peer1");
        let eid2 = EndpointId::from("dtn://peer2");

        descriptor.mark_sent(eid1.clone());
        descriptor.mark_sent(eid2.clone());

        assert!(descriptor.already_sent.contains(&eid1));
        assert!(descriptor.already_sent.contains(&eid2));
        assert_eq!(descriptor.already_sent.len(), 2);
    }

    #[test]
    fn test_mark_sent_duplicate() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        let eid = EndpointId::from("dtn://peer1");
        descriptor.mark_sent(eid.clone());
        descriptor.mark_sent(eid.clone()); // Duplicate

        assert!(descriptor.already_sent.contains(&eid));
        assert_eq!(descriptor.already_sent.len(), 1); // Should still be 1
    }

    #[test]
    fn test_get_already_sent() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        let eid1 = EndpointId::from("dtn://peer1");
        let eid2 = EndpointId::from("dtn://peer2");

        descriptor.mark_sent(eid1.clone());
        descriptor.mark_sent(eid2.clone());

        let already_sent = descriptor.get_already_sent();
        assert!(already_sent.contains(&eid1));
        assert!(already_sent.contains(&eid2));
        assert_eq!(already_sent.len(), 2);
    }

    #[test]
    fn test_has_been_sent_to() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        let eid1 = EndpointId::from("dtn://peer1");
        let eid2 = EndpointId::from("dtn://peer2");

        descriptor.mark_sent(eid1.clone());

        assert!(descriptor.has_been_sent_to(&eid1));
        assert!(!descriptor.has_been_sent_to(&eid2));
    }

    #[test]
    fn test_increment_forwarding_attempts() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        assert_eq!(descriptor.get_forwarding_attempts(), 0);

        descriptor.increment_forwarding_attempts();
        assert_eq!(descriptor.get_forwarding_attempts(), 1);

        descriptor.increment_forwarding_attempts();
        assert_eq!(descriptor.get_forwarding_attempts(), 2);
    }

    #[test]
    fn test_get_forwarding_attempts() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let descriptor = BundleDescriptor::new(bundle);

        assert_eq!(descriptor.get_forwarding_attempts(), 0);
    }

    #[test]
    fn test_is_ready_for_forwarding_valid() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let descriptor = BundleDescriptor::new(bundle);

        assert!(descriptor.is_ready_for_forwarding(5));
        assert!(descriptor.is_ready_for_forwarding(1));
    }

    #[test]
    fn test_is_ready_for_forwarding_max_attempts_reached() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        descriptor.increment_forwarding_attempts();
        descriptor.increment_forwarding_attempts();
        descriptor.increment_forwarding_attempts();

        assert!(!descriptor.is_ready_for_forwarding(3));
        assert!(!descriptor.is_ready_for_forwarding(2));
        assert!(descriptor.is_ready_for_forwarding(4));
    }

    #[test]
    fn test_is_ready_for_forwarding_expired_bundle() {
        // Create a bundle with very short lifetime
        let mut bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        bundle.primary.lifetime = 1; // 1 second

        let descriptor = BundleDescriptor::new(bundle);

        // Wait for bundle to expire
        thread::sleep(Duration::from_secs(2));

        assert!(!descriptor.is_ready_for_forwarding(5));
    }

    #[test]
    fn test_get_bundle_id() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let descriptor = BundleDescriptor::new(bundle.clone());

        let bundle_id = descriptor.get_bundle_id();
        let expected_id = format!(
            "{}-{}",
            bundle.primary.source, bundle.primary.creation_timestamp
        );

        assert_eq!(bundle_id, expected_id);
    }

    #[test]
    fn test_get_bundle_id_different_bundles() {
        let bundle1 = Bundle::new("dtn://src1", "dtn://dest", b"test1".to_vec());
        let bundle2 = Bundle::new("dtn://src2", "dtn://dest", b"test2".to_vec());

        let descriptor1 = BundleDescriptor::new(bundle1);
        let descriptor2 = BundleDescriptor::new(bundle2);

        assert_ne!(descriptor1.get_bundle_id(), descriptor2.get_bundle_id());
    }

    #[test]
    fn test_bundle_descriptor_clone() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        let eid = EndpointId::from("dtn://peer1");
        descriptor.mark_sent(eid.clone());
        descriptor.increment_forwarding_attempts();

        let cloned = descriptor.clone();

        assert_eq!(
            descriptor.bundle.primary.source,
            cloned.bundle.primary.source
        );
        assert_eq!(descriptor.forwarding_attempts, cloned.forwarding_attempts);
        assert_eq!(descriptor.created_at, cloned.created_at);
        assert!(cloned.already_sent.contains(&eid));
    }

    #[test]
    fn test_bundle_descriptor_debug() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let descriptor = BundleDescriptor::new(bundle);

        let debug_str = format!("{descriptor:?}");
        assert!(debug_str.contains("BundleDescriptor"));
        assert!(debug_str.contains("dtn://src"));
        assert!(debug_str.contains("dtn://dest"));
    }

    #[test]
    fn test_created_at_timestamp() {
        let bundle1 = Bundle::new("dtn://src", "dtn://dest", b"test1".to_vec());
        let descriptor1 = BundleDescriptor::new(bundle1);

        thread::sleep(Duration::from_millis(10));

        let bundle2 = Bundle::new("dtn://src", "dtn://dest", b"test2".to_vec());
        let descriptor2 = BundleDescriptor::new(bundle2);

        assert!(descriptor2.created_at >= descriptor1.created_at);
    }

    #[test]
    fn test_complex_forwarding_scenario() {
        let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
        let mut descriptor = BundleDescriptor::new(bundle);

        // Mark several peers as sent
        descriptor.mark_sent(EndpointId::from("dtn://peer1"));
        descriptor.mark_sent(EndpointId::from("dtn://peer2"));
        descriptor.mark_sent(EndpointId::from("dtn://peer3"));

        // Increment forwarding attempts
        descriptor.increment_forwarding_attempts();
        descriptor.increment_forwarding_attempts();

        // Test state
        assert_eq!(descriptor.already_sent.len(), 3);
        assert_eq!(descriptor.forwarding_attempts, 2);
        assert!(descriptor.has_been_sent_to(&EndpointId::from("dtn://peer1")));
        assert!(descriptor.has_been_sent_to(&EndpointId::from("dtn://peer2")));
        assert!(descriptor.has_been_sent_to(&EndpointId::from("dtn://peer3")));
        assert!(!descriptor.has_been_sent_to(&EndpointId::from("dtn://peer4")));
        assert!(descriptor.is_ready_for_forwarding(5));
        assert!(!descriptor.is_ready_for_forwarding(2));
    }
}
