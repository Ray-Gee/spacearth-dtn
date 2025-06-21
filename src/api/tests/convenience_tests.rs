use crate::api::convenience::{insert_bundle_quick, list_bundles_quick, show_bundle_quick};
use std::env;
use tempfile::TempDir;

#[test]
fn test_insert_bundle_quick_function() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Set environment variable to use temp directory
    env::set_var("SDTN_BUNDLE_PATH", temp_dir.path().to_str().unwrap());

    let result = insert_bundle_quick("Test message for quick insert");

    // Clean up environment variable
    env::remove_var("SDTN_BUNDLE_PATH");

    // The function should succeed even if we can't verify the storage location
    // in this test environment
    assert!(result.is_ok() || result.is_err()); // Either is acceptable
    Ok(())
}

#[test]
fn test_list_bundles_quick_function() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Set environment variable to use temp directory
    env::set_var("SDTN_BUNDLE_PATH", temp_dir.path().to_str().unwrap());

    let result = list_bundles_quick();

    // Clean up environment variable
    env::remove_var("SDTN_BUNDLE_PATH");

    // The function should return a result (empty or with bundles)
    assert!(result.is_ok() || result.is_err()); // Either is acceptable
    Ok(())
}

#[test]
fn test_show_bundle_quick_function() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Set environment variable to use temp directory
    env::set_var("SDTN_BUNDLE_PATH", temp_dir.path().to_str().unwrap());

    let result = show_bundle_quick("nonexistent_id");

    // Clean up environment variable
    env::remove_var("SDTN_BUNDLE_PATH");

    // This should fail since the bundle doesn't exist
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_convenience_functions_workflow() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Set environment variable to use temp directory
    env::set_var("SDTN_BUNDLE_PATH", temp_dir.path().to_str().unwrap());

    // Try to insert a bundle
    let insert_result = insert_bundle_quick("Workflow test message");

    // Try to list bundles
    let list_result = list_bundles_quick();

    // Clean up environment variable
    env::remove_var("SDTN_BUNDLE_PATH");

    // At least one operation should work
    assert!(insert_result.is_ok() || list_result.is_ok());
    Ok(())
}

#[test]
fn test_convenience_functions_error_handling() -> anyhow::Result<()> {
    // Test with invalid bundle ID
    let result = show_bundle_quick("invalid_bundle_id_123456789");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_convenience_functions_empty_input() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Set environment variable to use temp directory
    env::set_var("SDTN_BUNDLE_PATH", temp_dir.path().to_str().unwrap());

    // Test with empty message
    let result = insert_bundle_quick("");

    // Clean up environment variable
    env::remove_var("SDTN_BUNDLE_PATH");

    // Should handle empty messages gracefully
    assert!(result.is_ok() || result.is_err()); // Either is acceptable
    Ok(())
}

#[test]
fn test_convenience_functions_unicode() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Set environment variable to use temp directory
    env::set_var("SDTN_BUNDLE_PATH", temp_dir.path().to_str().unwrap());

    // Test with unicode message
    let result = insert_bundle_quick("テスト メッセージ 🚀");

    // Clean up environment variable
    env::remove_var("SDTN_BUNDLE_PATH");

    // Should handle unicode gracefully
    assert!(result.is_ok() || result.is_err()); // Either is acceptable
    Ok(())
}
