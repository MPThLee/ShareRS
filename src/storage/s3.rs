use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use sha2::Digest;

use super::{Storage, StorageError, UploadFileData};

pub struct S3 {
    bucket: Bucket,
}

impl S3 {
    pub fn new<S: Into<String>>(
        bucket: S,
        region: S,
        url: S,
        access: S,
        secret: S,
    ) -> Result<Self, StorageError> {
        let region = region.into();
        let bucket = Bucket::new(
            &bucket.into(),
            if region.as_str() == "r2" {
                Region::R2 {
                    account_id: url.into(),
                }
            } else {
                Region::Custom {
                    region: region,
                    endpoint: url.into(),
                }
            },
            Credentials::new(Some(&access.into()), Some(&secret.into()), None, None, None)
                .map_err(|_| {
                    StorageError::S3Error("Error while creating credentials".to_string())
                })?,
        )
        .map_err(|_| StorageError::S3Error("Error while creating Bucket instance".to_string()))?;
        Ok(S3 { bucket })
    }
}

#[async_trait]
impl Storage for S3 {
    async fn upload(
        &self,
        file_name: &str,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, StorageError> {
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&file_bytes));

        self.bucket
            .put_object(file_name, &file_bytes)
            .await
            .map_err(|_| StorageError::S3Error("Error while uploading file to S3".to_string()))?;

        Ok(UploadFileData {
            file_name: file_name.to_string(),
            content_length: file_bytes.len() as u32,
            content_sha512,
            timestamp: Utc::now().timestamp() as u64,
        })
    }

    async fn delete(&self, file_name: &str) -> Result<(), StorageError> {
        self.bucket
            .delete_object(&file_name)
            .await
            .map_err(|_| StorageError::S3Error("Error while deleting file on S3".to_string()))?;

        Ok(())
    }

    async fn get(&self, file_name: &str) -> Result<Bytes, StorageError> {
        let file =
            self.bucket.get_object(&file_name).await.map_err(|_| {
                StorageError::S3Error("Error while get file from S3".to_string())
            })?;
        Ok(file.bytes().clone())
    }

    async fn exists(&self, file_name: &str) -> bool {
        self.bucket.head_object(file_name).await.is_ok()
    }
}
