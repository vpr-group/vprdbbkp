use anyhow::{Context, Result};
use opendal::layers::LoggingLayer;
use opendal::services;
use opendal::Entry;
use opendal::Operator;

use super::configs::StorageConfig;

pub struct Storage {
    operator: Operator,
    storage_config: StorageConfig,
}

impl Storage {
    pub async fn new(storage_config: StorageConfig) -> Result<Self> {
        let operator = match storage_config.clone() {
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
                let builder = services::Fs::default();
                let operator = Operator::new(builder)?.layer(LoggingLayer::default());
                operator.finish()
            }
        };

        return Ok(Storage {
            operator,
            storage_config,
        });
    }

    pub async fn list(&self) -> Result<Vec<Entry>> {
        let prefix = match self.storage_config.clone() {
            StorageConfig::S3(config) => format!("{}", config.prefix.unwrap_or("".into())),
            StorageConfig::Local(config) => format!("{}", config.prefix.unwrap_or("".into())),
        };

        let entries = self
            .operator
            .list_with(&prefix)
            .recursive(true)
            .await
            .context(format!("Failed to list dumps in"))?;

        Ok(entries)
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
}
