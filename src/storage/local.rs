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
    async fn upload(
        &self,
        file_name: &str,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, StorageError> {
        let path = std::path::Path::new(&self.file_path).join(file_name.replace("../", ""));
        std::fs::create_dir_all(path.parent().ok_or(StorageError::InvalidFilename)?)?;
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&file_bytes));

        std::fs::write(path, &*file_bytes)?;
        Ok(UploadFileData {
            file_name: file_name.to_string(),
            content_length: file_bytes.len() as u32,
            content_sha512,
            timestamp: Utc::now().timestamp() as u64,
        })
    }

    async fn delete(&self, file_name: &str) -> Result<(), StorageError> {
        let path = std::path::Path::new(&self.file_path).join(file_name.replace("../", ""));
        std::fs::remove_file(path)?;

        Ok(())
    }

    async fn get(&self, file_name: &str) -> Result<Bytes, StorageError> {
        let path = std::path::Path::new(&self.file_path).join(file_name.replace("../", ""));
        Ok(Bytes::from(std::fs::read(path)?))
    }

    async fn exists(&self, file_name: &str) -> bool {
        let path = std::path::Path::new(&self.file_path).join(file_name.replace("../", ""));
        path.exists()
    }
}
