use sdtn::consts::BUNDLES_ADVANCED_DIR;
use sdtn::{convenience, BundleStatus, DtnNode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 SpaceArth DTN Advanced Usage Example");

    // Method 1: Using the full DtnNode API with default settings
    println!("\n📋 Method 1: Using DtnNode API (default settings)");
    let node = DtnNode::new()?;

    // Insert multiple bundles
    let messages = [
        "First message",
        "Second message with special chars: 🚀🌍",
        "Third message with numbers: 12345",
        "Fourth message - very long message that demonstrates how the system handles longer content",
    ];

    for (i, msg) in messages.iter().enumerate() {
        println!("  Inserting bundle {}: {}", i + 1, msg);
        node.insert_bundle(msg.to_string()).await?;
    }

    // Method 1b: Using DtnNode API with custom store path
    println!("\n📋 Method 1b: Using DtnNode API (custom store path)");
    let custom_node = DtnNode::with_store_path(BUNDLES_ADVANCED_DIR)?;
    custom_node
        .insert_bundle("Message in custom store".to_string())
        .await?;

    // Get detailed status for a specific bundle
    let bundles = node.list_bundles()?;
    if let Some(first_id) = bundles.first() {
        println!("\n📄 Detailed status for bundle: {}", first_id);
        let status = node.get_bundle_status(Some(first_id))?;
        match status {
            BundleStatus::Single { id, bundle } => {
                println!("  ID: {}", id);
                println!("  Source: {}", bundle.primary.source);
                println!("  Destination: {}", bundle.primary.destination);
                println!("  Creation Time: {}", bundle.primary.creation_timestamp);
                println!("  Lifetime: {} seconds", bundle.primary.lifetime);
                println!("  Expired: {}", bundle.is_expired());
                println!("  Message: {}", String::from_utf8_lossy(&bundle.payload));
            }
            _ => unreachable!(),
        }
    }

    // Method 2: Using convenience functions (always use default path)
    println!("\n📋 Method 2: Using convenience functions");

    // Quick insert
    convenience::insert_bundle_quick("Quick message from convenience function").await?;

    // Quick list
    let quick_bundles = convenience::list_bundles_quick()?;
    println!(
        "  Found {} bundles using convenience function",
        quick_bundles.len()
    );

    // Quick show (if we have bundles)
    if let Some(bundle_id) = quick_bundles.first() {
        let partial_id = &bundle_id[..8]; // Use first 8 characters
        match convenience::show_bundle_quick(partial_id) {
            Ok(bundle) => {
                println!(
                    "  Quick show for {}: {}",
                    partial_id,
                    String::from_utf8_lossy(&bundle.payload)
                );
            }
            Err(e) => {
                println!("  Quick show failed: {}", e);
            }
        }
    }

    // Method 3: Error handling demonstration
    println!("\n📋 Method 3: Error handling demonstration");

    // Try to show a non-existent bundle
    match node.show_bundle("nonexistent") {
        Ok(_) => println!("  Unexpected: Found non-existent bundle"),
        Err(e) => println!("  Expected error for non-existent bundle: {}", e),
    }

    // Method 4: Status summary
    println!("\n📋 Method 4: Status summary");
    let summary = node.get_bundle_status(None)?;
    match summary {
        BundleStatus::Summary {
            active,
            expired,
            total,
        } => {
            println!("  📊 Bundle Summary:");
            println!("    ✅ Active: {}", active);
            println!("    ⏰ Expired: {}", expired);
            println!("    📦 Total: {}", total);

            if expired > 0 {
                println!("  🧹 Cleaning up expired bundles...");
                node.cleanup_expired()?;

                // Check status after cleanup
                let after_cleanup = node.get_bundle_status(None)?;
                if let BundleStatus::Summary {
                    active: new_active,
                    expired: new_expired,
                    total: new_total,
                } = after_cleanup
                {
                    println!("  📊 After cleanup:");
                    println!("    ✅ Active: {}", new_active);
                    println!("    ⏰ Expired: {}", new_expired);
                    println!("    📦 Total: {}", new_total);
                }
            }
        }
        _ => unreachable!(),
    }

    // Method 5: Using Default trait
    println!("\n📋 Method 5: Using Default trait");
    let default_node = DtnNode::default();
    default_node
        .insert_bundle("Message from default instance".to_string())
        .await?;

    println!("\n✅ Advanced usage example completed successfully!");
    Ok(())
}
