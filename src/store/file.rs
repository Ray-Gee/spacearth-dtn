use crate::bundle::Bundle;
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
