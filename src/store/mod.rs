pub mod bundle_descriptor;
pub mod file;

pub use bundle_descriptor::BundleDescriptor;
pub use file::BundleStore;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod existing_tests {
    use super::*;

    #[test]
    fn test_bundle_store_reexport() {
        // Test that the re-export works correctly
        let store_type_id = std::any::TypeId::of::<BundleStore>();
        let file_store_type_id = std::any::TypeId::of::<file::BundleStore>();

        assert_eq!(store_type_id, file_store_type_id);
    }

    #[test]
    fn test_module_accessibility() {
        // Test that the file module is accessible
        use crate::store::file::BundleStore as FileBundleStore;

        let _ = std::any::TypeId::of::<FileBundleStore>();
        let _ = std::any::TypeId::of::<BundleStore>();
    }

    #[test]
    fn test_type_names() {
        let bundle_store_name = std::any::type_name::<BundleStore>();
        let file_bundle_store_name = std::any::type_name::<file::BundleStore>();

        assert_eq!(bundle_store_name, file_bundle_store_name);
        assert!(bundle_store_name.contains("BundleStore"));
    }
}
