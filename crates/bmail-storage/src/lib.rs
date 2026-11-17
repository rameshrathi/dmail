use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum StorageError {
    #[error("Object not found at URI: {0}")]
    NotFound(String),

    #[error("Integrity check failed: expected hash {expected}, got {actual}")]
    IntegrityCheckFailed { expected: String, actual: String },

    #[error("Storage I/O error: {0}")]
    IoError(String),
}

/// Abstract interface for decentralized / content-addressed storage.
pub trait StorageAdapter: Send + Sync {
    /// Put raw ciphertext bytes into storage and return a content URI (e.g. `ipfs://<hash>`).
    fn put(&self, data: &[u8]) -> Result<String, StorageError>;

    /// Retrieve raw ciphertext bytes by URI.
    fn get(&self, uri: &str) -> Result<Vec<u8>, StorageError>;
}

/// In-memory content-addressed storage adapter (for local dev, tests, and mock IPFS).
#[derive(Default)]
pub struct MemoryStorageAdapter {
    store: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemoryStorageAdapter {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl StorageAdapter for MemoryStorageAdapter {
    fn put(&self, data: &[u8]) -> Result<String, StorageError> {
        let hash = bmail_crypto::sha256_digest(data);
        let hash_hex = hex_string(&hash);
        let uri = format!("ipfs://{}", hash_hex);

        let mut lock = self.store.lock().unwrap();
        lock.insert(uri.clone(), data.to_vec());
        Ok(uri)
    }

    fn get(&self, uri: &str) -> Result<Vec<u8>, StorageError> {
        let lock = self.store.lock().unwrap();
        match lock.get(uri) {
            Some(bytes) => Ok(bytes.clone()),
            None => Err(StorageError::NotFound(uri.to_string())),
        }
    }
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_put_get() {
        let storage = MemoryStorageAdapter::new();
        let sample_ciphertext = b"Encrypted Ciphertext Payload Blob";
        let uri = storage.put(sample_ciphertext).unwrap();
        assert!(uri.starts_with("ipfs://"));

        let retrieved = storage.get(&uri).unwrap();
        assert_eq!(sample_ciphertext, retrieved.as_slice());

        assert!(storage.get("ipfs://nonexistent").is_err());
    }
}
