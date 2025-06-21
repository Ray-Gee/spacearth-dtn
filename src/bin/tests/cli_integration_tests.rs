use sdtn::api::DtnNode;
use sdtn::cli::*;
use tempfile::TempDir;

#[test]
fn test_handle_insert_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let result = handle_insert_command(&node, "test message".to_string());
    assert!(result.is_ok());

    // Verify the bundle was actually inserted
    let bundles = node.list_bundles()?;
    assert_eq!(bundles.len(), 1);
    Ok(())
}

#[test]
fn test_handle_list_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    // Test empty list
    let result = handle_list_command(&node);
    assert!(result.is_ok());

    // Test with bundles
    node.insert_bundle("test message".to_string())?;
    let result = handle_list_command(&node);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_show_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("test message".to_string())?;
    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap().clone();

    let result = handle_show_command(&node, bundle_id);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_status_command_single() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("test message".to_string())?;
    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap().clone();

    let result = handle_status_command(&node, Some(bundle_id));
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_status_command_summary() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("test message".to_string())?;

    let result = handle_status_command(&node, None);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_cleanup_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let result = handle_cleanup_command(&node);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_route_show_command() -> anyhow::Result<()> {
    let result = handle_route_show_command();
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_route_table_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let result = handle_route_table_command(&node);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_route_add_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let result = handle_route_add_command(
        &node,
        "dtn://dest".to_string(),
        "dtn://next".to_string(),
        "tcp".to_string(),
        10,
    );
    assert!(result.is_ok());

    // Verify the route was added
    let routes = node.get_all_routes()?;
    assert_eq!(routes.len(), 1);
    Ok(())
}

#[test]
fn test_handle_route_test_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("test message".to_string())?;
    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap().clone();

    let result = handle_route_test_command(&node, bundle_id);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_route_test_table_command() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("test message".to_string())?;
    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap().clone();

    let result = handle_route_test_table_command(&node, bundle_id);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_route_set_command() -> anyhow::Result<()> {
    let result = handle_route_set_command("epidemic".to_string());
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_cli_functions_with_various_inputs() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    // Test with different message types
    let messages = ["hello", "unicode: 🌍", "empty", ""];
    for msg in &messages {
        let result = handle_insert_command(&node, msg.to_string());
        assert!(result.is_ok());
    }

    // Test listing
    let result = handle_list_command(&node);
    assert!(result.is_ok());

    // Test showing first bundle
    let bundles = node.list_bundles()?;
    if !bundles.is_empty() {
        let result = handle_show_command(&node, bundles[0].clone());
        assert!(result.is_ok());
    }

    Ok(())
}
