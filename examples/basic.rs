use sdtn::{BundleStatus, DtnNode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create DTN CLI instance with default settings
    let node = DtnNode::new()?;

    // Insert a bundle
    println!("📦 Inserting a new bundle...");
    node.insert_bundle("Hello from SpaceArth DTN!".to_string())
        .await?;

    // List all bundles
    println!("📋 Listing all bundles...");
    let bundles = node.list_bundles()?;
    println!("Found {} bundles:", bundles.len());
    for id in &bundles {
        println!("  {}", id);
    }

    // Show details of a specific bundle
    if let Some(first_id) = bundles.first() {
        println!("📄 Showing details for bundle: {}", first_id);
        let bundle = node.show_bundle(first_id)?;
        println!("  Source: {}", bundle.primary.source);
        println!("  Destination: {}", bundle.primary.destination);
        println!("  Message: {}", String::from_utf8_lossy(&bundle.payload));
        println!("  Expired: {}", bundle.is_expired());
    }

    // Get status summary
    println!("📊 Getting status summary...");
    let status = node.get_bundle_status(None)?;
    match status {
        BundleStatus::Summary {
            active,
            expired,
            total,
        } => {
            println!("  Active bundles: {}", active);
            println!("  Expired bundles: {}", expired);
            println!("  Total bundles: {}", total);
        }
        _ => unreachable!(),
    }

    // Clean up expired bundles
    println!("🧹 Cleaning up expired bundles...");
    node.cleanup_expired()?;

    println!("✅ Basic usage example completed!");
    Ok(())
}
