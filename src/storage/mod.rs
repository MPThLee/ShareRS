use async_trait::async_trait;
use bytes::Bytes;
use thiserror::Error;

mod local;
mod s3;

pub use self::s3::S3;
pub use local::Local;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("S3 error: {0}")]
    S3Error(String),
    #[error("File system error in storage: {0}")]
    FileSystemError(#[from] std::io::Error),
    #[error("Invalid Filename")]
    InvalidFilename,
}

#[derive(Debug, Clone)]
pub struct UploadFileData {
    pub file_name: String,
    pub content_length: u32,
    pub content_sha512: String,
    pub timestamp: u64,
}

#[async_trait]
pub trait Storage {
    async fn put(&self, file_name: &str, file_bytes: Bytes)
        -> Result<UploadFileData, StorageError>;

    async fn delete(&self, key: &str) -> Result<(), StorageError>;

    async fn get(&self, key: &str) -> Result<Bytes, StorageError>;

    async fn exists(&self, key: &str) -> bool;
}
