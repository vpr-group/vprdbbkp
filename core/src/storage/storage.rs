use std::borrow::Borrow;
use std::time::Duration;
use std::time::SystemTime;

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use chrono::DateTime;
use chrono::Utc;
use opendal::layers::LoggingLayer;
use opendal::services;
use opendal::Entry;
use opendal::Operator;

use crate::utils::extract_timestamp_from_filename;

use super::configs::StorageConfig;

pub struct Storage {
    operator: Operator,
    storage_config: StorageConfig,
}

impl Storage {
    pub async fn new<B>(storage_config: B) -> Result<Self>
    where
        B: Borrow<StorageConfig>,
    {
        let borrowed_storage_config = storage_config.borrow();

        let operator = match borrowed_storage_config {
            StorageConfig::S3(config) => {
                let builder = services::S3::default()
                    .bucket(&config.bucket)
                    .region(&config.region)
                    .endpoint(&config.endpoint)
                    .access_key_id(&config.access_key)
                    .secret_access_key(&config.secret_key);

                let operator = Operator::new(builder)?.layer(LoggingLayer::default());
                operator.finish()
            }
            StorageConfig::Local(config) => {
                let builder = services::Fs::default().root(config.root.to_str().unwrap_or(""));
                let operator = Operator::new(builder)?.layer(LoggingLayer::default());
                operator.finish()
            }
        };

        return Ok(Storage {
            operator,
            storage_config: borrowed_storage_config.clone(),
        });
    }

    pub fn get_prefix(&self) -> String {
        let prefix = match self.storage_config.clone() {
            StorageConfig::S3(config) => format!("{}", config.prefix.unwrap_or("".into())),
            StorageConfig::Local(config) => format!("{}", config.prefix.unwrap_or("".into())),
        };

        prefix
    }

    pub fn get_filename_from_path(&self, path: &str) -> String {
        let prefix = self.get_prefix();

        if path.starts_with(&format!("{}/", prefix)) {
            // Remove the prefix and the following slash
            path[prefix.len() + 1..].to_string()
        } else {
            // If path doesn't start with prefix, return the original path
            path.to_string()
        }
    }

    pub async fn list(&self) -> Result<Vec<Entry>> {
        let prefix = self.get_prefix();

        let entries = self
            .operator
            .list_with(&prefix)
            .recursive(true)
            .limit(10000)
            .await
            .context(format!("Failed to list dumps in"))?;

        Ok(entries)
    }

    pub async fn write(&self, filename: &str, bytes: Bytes) -> Result<String> {
        let prefix = self.get_prefix();
        let path = format!("{}/{}", prefix, filename);

        self.operator
            .write(&path, bytes)
            .await
            .context(format!("Failed to write bytes"))?;

        Ok(path)
    }

    pub async fn read(&self, filename: &str) -> Result<Bytes> {
        let prefix = self.get_prefix();
        let path = format!("{}/{}", prefix, filename);

        let buffer = self
            .operator
            .read(&path)
            .await
            .context(format!("Failed to read file {}", path))?;

        Ok(Bytes::from(buffer.to_vec()))
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        self.operator
            .delete(&path)
            .await
            .context(format!("Failed to delete backup {}", path))?;

        Ok(())
    }

    pub async fn cleanup(&self, retention_days: u64, dry_run: bool) -> Result<(usize, u64)> {
        let backups = self.list().await?;

        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(retention_days * 86400))
            .ok_or_else(|| anyhow!("Failed to calculate cutoff date"))?;

        let cutoff_datetime: DateTime<Utc> = cutoff.into();

        let mut deleted_count = 0;
        let mut deleted_size = 0;

        for backup in backups {
            match extract_timestamp_from_filename(backup.name()) {
                Ok(timestamp) => {
                    if timestamp < cutoff_datetime {
                        let size = backup.metadata().content_length();
                        deleted_size += size;
                        deleted_count += 1;

                        if !dry_run {
                            self.delete(&backup.path()).await?;
                        }
                    }
                }
                Err(_) => {}
            };
        }

        Ok((deleted_count, deleted_size))
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::storage::configs::S3StorageConfig;
    use dotenv::dotenv;

    use super::*;

    #[tokio::test]
    async fn test_s3_list() {
        dotenv().ok();

        let storage = Storage::new(StorageConfig::S3(S3StorageConfig {
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string()),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            name: env::var("S3_CONFIG_NAME").unwrap_or_else(|_| "s3-test".to_string()),
            prefix: Some(env::var("S3_PREFIX").unwrap_or_default()),
        }))
        .await
        .expect("Unable to create storage");

        let entries = storage.list().await.expect("Unable to list dumps");

        println!("{:?}", entries);
    }

    #[tokio::test]
    async fn test_s3_cleanup() {
        dotenv().ok();

        let storage = Storage::new(StorageConfig::S3(S3StorageConfig {
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string()),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            name: env::var("S3_CONFIG_NAME").unwrap_or_else(|_| "s3-test".to_string()),
            prefix: None,
        }))
        .await
        .expect("Unable to create storage config");

        let (deleted_count, seleted_size) =
            storage.cleanup(1, false).await.expect("Unable to cleanup");

        assert!(deleted_count > 1);
        assert!(seleted_size > 1);
    }
}
