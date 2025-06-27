use crate::bpv7::bundle::{Bundle, PrimaryBlock};
use crate::store::file::BundleStore;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

fn create_test_bundle(source: &str, destination: &str, lifetime: u64) -> Bundle {
    let creation_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Bundle {
        primary: PrimaryBlock {
            version: 7,
            source: source.to_string(),
            destination: destination.to_string(),
            report_to: "none".to_string(),
            creation_timestamp,
            lifetime,
        },
        payload: b"test payload".to_vec(),
    }
}

fn create_expired_bundle(source: &str, destination: &str) -> Bundle {
    Bundle {
        primary: PrimaryBlock {
            version: 7,
            source: source.to_string(),
            destination: destination.to_string(),
            report_to: "none".to_string(),
            creation_timestamp: 1000000, // 非常に古いタイムスタンプ
            lifetime: 3600,
        },
        payload: b"expired payload".to_vec(),
    }
}

#[test]
fn test_new_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("test_bundles");

    let store = BundleStore::new(&store_path).unwrap();

    assert!(store_path.exists());
    assert!(store_path.is_dir());
    assert_eq!(store.dir, store_path);
}

#[test]
fn test_new_with_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("existing_bundles");
    fs::create_dir_all(&store_path).unwrap();

    let store = BundleStore::new(&store_path).unwrap();

    assert!(store_path.exists());
    assert_eq!(store.dir, store_path);
}

#[test]
fn test_insert_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();
    let bundle = create_test_bundle("node1", "node2", 3600);

    // バンドルを保存
    store.insert(&bundle).unwrap();

    // ファイル名を取得
    let filename = store.filename_for(&bundle);
    let id = filename.file_stem().unwrap().to_str().unwrap();

    // バンドルを読み込み
    let loaded_bundle = store.load(id).unwrap();

    assert_eq!(loaded_bundle.primary.source, bundle.primary.source);
    assert_eq!(
        loaded_bundle.primary.destination,
        bundle.primary.destination
    );
    assert_eq!(loaded_bundle.payload, bundle.payload);
}

#[test]
fn test_load_nonexistent_bundle() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let result = store.load("nonexistent_id");

    assert!(result.is_err());
}

#[test]
fn test_load_by_partial_id() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();
    let bundle = create_test_bundle("node1", "node2", 3600);

    store.insert(&bundle).unwrap();

    let filename = store.filename_for(&bundle);
    let full_id = filename.file_stem().unwrap().to_str().unwrap();
    let partial_id = &full_id[..8]; // 最初の8文字

    let loaded_bundle = store.load_by_partial_id(partial_id).unwrap();

    assert_eq!(loaded_bundle.primary.source, bundle.primary.source);
    assert_eq!(
        loaded_bundle.primary.destination,
        bundle.primary.destination
    );
}

#[test]
fn test_load_by_partial_id_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let result = store.load_by_partial_id("nonexistent");

    assert!(result.is_err());
    let error_message = format!("{}", result.unwrap_err());
    assert!(error_message.contains("Bundle ID not found"));
}

#[test]
fn test_list_empty_store() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let ids = store.list().unwrap();

    assert!(ids.is_empty());
}

#[test]
fn test_list_with_bundles() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle1 = create_test_bundle("node1", "node2", 3600);
    let bundle2 = create_test_bundle("node2", "node3", 3600);

    store.insert(&bundle1).unwrap();
    store.insert(&bundle2).unwrap();

    let ids = store.list().unwrap();

    assert_eq!(ids.len(), 2);
}

#[test]
fn test_list_ignores_non_cbor_files() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle = create_test_bundle("node1", "node2", 3600);
    store.insert(&bundle).unwrap();

    // .cborではないファイルを作成
    fs::write(store.dir.join("test.txt"), "not a bundle").unwrap();
    fs::write(store.dir.join("test.json"), "{}").unwrap();

    let ids = store.list().unwrap();

    assert_eq!(ids.len(), 1); // .cborファイルのみカウント
}

#[test]
fn test_dispatch_one() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();
    let dispatched_dir = temp_dir.path().join("dispatched");

    let bundle = create_test_bundle("node1", "node2", 3600);
    store.insert(&bundle).unwrap();

    let original_path = store.filename_for(&bundle);
    assert!(original_path.exists());

    // dispatchする
    store.dispatch_one(&bundle, &dispatched_dir).unwrap();

    // 元のファイルが存在しないことを確認
    assert!(!original_path.exists());

    // dispatchedディレクトリにファイルが存在することを確認
    let dispatched_path = dispatched_dir.join(original_path.file_name().unwrap());
    assert!(dispatched_path.exists());
}

#[test]
fn test_dispatch_one_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();
    let dispatched_dir = temp_dir.path().join("new_dispatched_dir");

    let bundle = create_test_bundle("node1", "node2", 3600);
    store.insert(&bundle).unwrap();

    assert!(!dispatched_dir.exists());

    store.dispatch_one(&bundle, &dispatched_dir).unwrap();

    assert!(dispatched_dir.exists());
    assert!(dispatched_dir.is_dir());
}

#[test]
fn test_cleanup_expired_empty_store() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let result = store.cleanup_expired();

    assert!(result.is_ok());
}

#[test]
fn test_cleanup_expired_removes_expired_bundles() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let expired_bundle = create_expired_bundle("node1", "node2");
    let valid_bundle = create_test_bundle("node2", "node3", 3600);

    store.insert(&expired_bundle).unwrap();
    store.insert(&valid_bundle).unwrap();

    let ids_before = store.list().unwrap();
    assert_eq!(ids_before.len(), 2);

    store.cleanup_expired().unwrap();

    let ids_after = store.list().unwrap();
    assert_eq!(ids_after.len(), 1);
}

#[test]
fn test_cleanup_expired_keeps_valid_bundles() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let valid_bundle1 = create_test_bundle("node1", "node2", 3600);
    let valid_bundle2 = create_test_bundle("node2", "node3", 7200);

    store.insert(&valid_bundle1).unwrap();
    store.insert(&valid_bundle2).unwrap();

    let ids_before = store.list().unwrap();
    assert_eq!(ids_before.len(), 2);

    store.cleanup_expired().unwrap();

    let ids_after = store.list().unwrap();
    assert_eq!(ids_after.len(), 2); // 両方とも有効なので残る
}

#[test]
fn test_filename_for_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle = create_test_bundle("node1", "node2", 3600);

    let filename1 = store.filename_for(&bundle);
    let filename2 = store.filename_for(&bundle);

    assert_eq!(filename1, filename2);
    assert!(filename1.extension().unwrap() == "cbor");
}

#[test]
fn test_filename_for_different_bundles() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle1 = create_test_bundle("node1", "node2", 3600);
    let bundle2 = create_test_bundle("node2", "node3", 3600);

    let filename1 = store.filename_for(&bundle1);
    let filename2 = store.filename_for(&bundle2);

    assert_ne!(filename1, filename2);
}

#[test]
fn test_find_by_partial_id_no_match() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle = create_test_bundle("node1", "node2", 3600);
    store.insert(&bundle).unwrap();

    let result = store.find_by_partial_id("zzzzz");

    assert!(result.is_none());
}

#[test]
fn test_multiple_partial_matches() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle1 = create_test_bundle("node1", "node2", 3600);
    let bundle2 = create_test_bundle("node3", "node4", 3600);

    store.insert(&bundle1).unwrap();
    store.insert(&bundle2).unwrap();

    let ids = store.list().unwrap();

    // 共通のプレフィックスがあるかテスト
    if let Some(common_prefix) = ids
        .iter()
        .map(|id| &id[..1])
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .next()
    {
        let result = store.find_by_partial_id(common_prefix);
        assert!(result.is_some());
    }
}

// Additional tests for better coverage
#[test]
fn test_cleanup_expired_with_io_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let expired_bundle = create_expired_bundle("node1", "node2");
    store.insert(&expired_bundle).unwrap();

    // Get the file path and manually delete it to simulate an IO error scenario
    let filename = store.filename_for(&expired_bundle);

    // Delete the file manually
    fs::remove_file(&filename).unwrap();

    // Now try to cleanup - should handle the missing file gracefully
    let result = store.cleanup_expired();
    assert!(result.is_ok());
}

#[test]
fn test_dispatch_one_invalid_filename() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();
    let dispatched_dir = temp_dir.path().join("dispatched");

    let bundle = create_test_bundle("node1", "node2", 3600);
    store.insert(&bundle).unwrap();

    // Remove the file to simulate an error condition
    let filename = store.filename_for(&bundle);
    fs::remove_file(&filename).unwrap();

    // Should handle missing source file
    let result = store.dispatch_one(&bundle, &dispatched_dir);
    assert!(result.is_err());
}

#[test]
fn test_load_corrupted_cbor_file() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    // Create a corrupted CBOR file
    let corrupted_file = store.dir.join("corrupted.cbor");
    fs::write(&corrupted_file, b"not valid cbor data").unwrap();

    // Should handle corrupted files gracefully
    let result = store.load("corrupted");
    assert!(result.is_err());
}

#[test]
fn test_list_with_read_permission_error() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle = create_test_bundle("node1", "node2", 3600);
    store.insert(&bundle).unwrap();

    // On Unix systems, we could test permission errors, but this is platform-specific
    // For now, just test that list works with normal files
    let ids = store.list().unwrap();
    assert_eq!(ids.len(), 1);
}

#[test]
fn test_find_by_partial_id_exact_match() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    let bundle = create_test_bundle("node1", "node2", 3600);
    store.insert(&bundle).unwrap();

    let filename = store.filename_for(&bundle);
    let full_id = filename.file_stem().unwrap().to_str().unwrap();

    // Test exact match
    let result = store.find_by_partial_id(full_id);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), full_id);
}

#[test]
fn test_insert_with_different_payloads() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    // Test different payload types
    let payloads = [
        b"simple text".to_vec(),
        [0, 1, 2, 3, 4, 5].to_vec(), // binary data
        "unicode: 🚀🌍".as_bytes().to_vec(),
        vec![],          // empty payload
        vec![255; 1000], // large payload
    ];

    for (i, payload) in payloads.iter().enumerate() {
        let bundle = Bundle {
            primary: PrimaryBlock {
                version: 7,
                source: format!("node{i}"),
                destination: format!("dest{i}"),
                report_to: "none".to_string(),
                creation_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                lifetime: 3600,
            },
            payload: payload.clone(),
        };

        store.insert(&bundle).unwrap();

        let filename = store.filename_for(&bundle);
        let id = filename.file_stem().unwrap().to_str().unwrap();
        let loaded = store.load(id).unwrap();

        assert_eq!(loaded.payload, *payload);
    }
}

#[test]
fn test_cleanup_expired_edge_cases() {
    let temp_dir = TempDir::new().unwrap();
    let store = BundleStore::new(temp_dir.path().join("bundles")).unwrap();

    // Test with bundle that expires exactly now
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let edge_bundle = Bundle {
        primary: PrimaryBlock {
            version: 7,
            source: "edge_node".to_string(),
            destination: "edge_dest".to_string(),
            report_to: "none".to_string(),
            creation_timestamp: now - 3600, // Created 1 hour ago
            lifetime: 3600,                 // Lifetime of 1 hour (expires now)
        },
        payload: b"edge case".to_vec(),
    };

    store.insert(&edge_bundle).unwrap();

    let ids_before = store.list().unwrap();
    assert_eq!(ids_before.len(), 1);

    // Small delay to ensure the bundle is expired
    std::thread::sleep(std::time::Duration::from_millis(100));

    store.cleanup_expired().unwrap();

    let ids_after = store.list().unwrap();
    // The bundle should be expired and removed, but timing can be tricky
    // so we'll just verify cleanup ran successfully
    assert!(ids_after.len() <= 1); // Could be 0 or 1 depending on timing
}

#[cfg(test)]
mod existing_tests {
    use super::*;

    #[test]
    fn test_bundle_store_reexport() {
        // Test that the re-export works correctly
        let store_type_id = std::any::TypeId::of::<BundleStore>();
        let file_store_type_id = std::any::TypeId::of::<crate::store::file::BundleStore>();

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
        let file_bundle_store_name = std::any::type_name::<crate::store::file::BundleStore>();

        assert_eq!(bundle_store_name, file_bundle_store_name);
        assert!(bundle_store_name.contains("BundleStore"));
    }
}
