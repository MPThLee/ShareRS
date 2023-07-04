use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use sha2::Digest;

use super::{Storage, StorageError, UploadFileData};

pub struct Local {
    file_path: String,
}

impl Local {
    pub fn new<S: Into<String>>(file_path: S) -> Self {
        Local {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl Storage for Local {
    async fn put(&self, key: &str, bytes: Bytes) -> Result<UploadFileData, StorageError> {
        let path = std::path::Path::new(&self.file_path).join(key.replace("../", ""));
        let create = tokio::fs::create_dir_all(path.parent().ok_or(StorageError::InvalidFilename)?);
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&bytes));

        create.await?;
        tokio::fs::write(path, &*bytes).await?;
        Ok(UploadFileData {
            file_name: key.to_string(),
            content_length: bytes.len() as u32,
            content_sha512,
            timestamp: Utc::now().timestamp() as u64,
        })
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = std::path::Path::new(&self.file_path).join(key.replace("../", ""));
        std::fs::remove_file(path)?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        let path = std::path::Path::new(&self.file_path).join(key.replace("../", ""));
        Ok(Bytes::from(std::fs::read(path)?))
    }

    async fn exists(&self, key: &str) -> bool {
        let path = std::path::Path::new(&self.file_path).join(key.replace("../", ""));
        path.exists()
    }
}
