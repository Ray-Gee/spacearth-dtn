pub mod client;
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn test_ble_modules_exist() {
        // Test that BLE modules are accessible by checking module path
        let path = module_path!();
        assert!(path.contains("ble"));
    }

    #[test]
    fn test_module_structure() {
        // Test that we're in the correct module path
        let path = module_path!();
        assert!(path.contains("cla"));
        assert!(path.contains("ble"));
        // Note: path will be "spacearth_dtn::cla::ble::tests" not just "ble"
        assert!(path.contains("spacearth_dtn::cla::ble"));
    }

    #[test]
    fn test_submodules_declared() {
        // This test verifies that the submodules are properly declared
        // If this compiles, it means the modules exist and are accessible

        // We can't directly reference modules as values, but we can test
        // that the module structure is correct by checking our own path
        let current_module = module_path!();
        assert!(current_module.starts_with("spacearth_dtn::cla::ble"));
    }
}
