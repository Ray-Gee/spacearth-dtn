use super::DtnNode;
use crate::bpv7::bundle::Bundle;

/// Quick bundle insertion using default settings
pub async fn insert_bundle_quick(message: &str) -> anyhow::Result<()> {
    let node = DtnNode::new()?;
    node.insert_bundle(message.to_string()).await
}

/// Quick bundle listing using default settings
pub fn list_bundles_quick() -> anyhow::Result<Vec<String>> {
    let node = DtnNode::new()?;
    node.list_bundles()
}

/// Quick bundle show using default settings
pub fn show_bundle_quick(partial_id: &str) -> anyhow::Result<Bundle> {
    let node = DtnNode::new()?;
    node.show_bundle(partial_id)
}
