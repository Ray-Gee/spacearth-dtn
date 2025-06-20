use crate::bpv7::bundle::Bundle;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct BundleStore {
    dir: PathBuf,
}

impl BundleStore {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let dir = path.into();
        fs::create_dir_all(&dir)?;
        Ok(BundleStore { dir })
    }

    fn filename_for(&self, bundle: &Bundle) -> PathBuf {
        let id_str = format!(
            "{}:{}:{}:{}",
            bundle.primary.version,
            bundle.primary.source,
            bundle.primary.destination,
            bundle.primary.creation_timestamp
        );
        let hash = Sha256::digest(id_str.as_bytes());
        self.dir.join(format!("{:x}.cbor", hash))
    }

    pub fn insert(&self, bundle: &Bundle) -> Result<()> {
        let path = self.filename_for(bundle);
        let encoded = serde_cbor::to_vec(bundle)?;
        fs::write(&path, encoded)?;
        println!(
            "Bundle saved to {} (ID: {})",
            path.display(),
            path.file_stem().unwrap().to_string_lossy()
        );
        Ok(())
    }

    pub fn load(&self, id_hash: &str) -> Result<Bundle> {
        let path = self.dir.join(format!("{id_hash}.cbor"));
        let data = fs::read(path)?;
        let bundle = serde_cbor::from_slice(&data)?;
        Ok(bundle)
    }

    pub fn load_by_partial_id(&self, partial: &str) -> Result<Bundle> {
        if let Some(full_id) = self.find_by_partial_id(partial) {
            self.load(&full_id)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Bundle ID not found").into())
        }
    }

    fn find_by_partial_id(&self, partial: &str) -> Option<String> {
        match self.list() {
            Ok(ids) => ids.into_iter().find(|id| id.starts_with(partial)),
            Err(_) => None,
        }
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let mut result = vec![];
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("cbor") {
                if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    result.push(name.to_string());
                }
            }
        }
        Ok(result)
    }

    pub fn dispatch_one(&self, bundle: &Bundle, dispatched_dir: &Path) -> Result<()> {
        let src = self.filename_for(bundle);
        let dst = dispatched_dir.join(
            src.file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?,
        );
        fs::create_dir_all(dispatched_dir)?;
        fs::rename(src, dst)?;
        Ok(())
    }

    pub fn cleanup_expired(&self) -> Result<()> {
        let ids = self.list()?;
        println!("🔍 Found {} bundle IDs: {:?}", ids.len(), ids);
        if ids.is_empty() {
            println!("📦 No bundles found");
            return Ok(());
        }

        for id in ids {
            let bundle = match self.load_by_partial_id(&id) {
                Ok(bundle) => bundle,
                Err(e) => {
                    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                        if io_err.kind() == std::io::ErrorKind::NotFound {
                            continue;
                        }
                    }
                    return Err(e);
                }
            };

            if bundle.is_expired() {
                let path = self.dir.join(format!("{id}.cbor"));
                println!("🔍 Attempting to remove: {:?}", path);
                match std::fs::remove_file(&path) {
                    Ok(_) => println!("🗑️  Removed expired bundle: {id}"),
                    Err(e) => {
                        println!("❌ Failed to remove: {:?} - {:?}", path, e);
                        if e.kind() != std::io::ErrorKind::NotFound {
                            return Err(e.into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpv7::bundle::{Bundle, PrimaryBlock};
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
}
