use crate::bundle::Bundle;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

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

    pub fn dispatch_all(&self, dispatched_dir: &Path) -> Result<()> {
        fs::create_dir_all(dispatched_dir)?;
        for id in self.list()? {
            let bundle = self.load(&id)?;
            println!("📤 Dispatching bundle: {}", id);

            // ⚠️ Actual transmission is still a dummy (only logging for now)
            println!("  To: {}", bundle.primary.destination);

            // 移動する
            let from = self.dir.join(format!("{id}.cbor"));
            let to = dispatched_dir.join(format!("{id}.cbor"));
            fs::rename(from, to)?;
        }
        Ok(())
    }
}
