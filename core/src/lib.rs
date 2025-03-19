use std::borrow::Borrow;

use anyhow::Result;

pub mod databases;
pub mod folders;
pub mod platform;
pub mod storage;
pub mod utils;

use databases::{backup_source, configs::SourceConfig, restore_source};
use opendal::Entry;

use storage::{configs::StorageConfig, storage::Storage};
use utils::get_filename;
pub use utils::{format_timestamp, get_backup_key};

pub async fn backup<SO, ST>(source_config: SO, storage_config: ST) -> Result<String>
where
    SO: Borrow<SourceConfig>,
    ST: Borrow<StorageConfig>,
{
    let borrowed_source_config = source_config.borrow();
    let borrowed_storage_config = storage_config.borrow();

    let bytes = backup_source(borrowed_source_config).await?;
    let storage = Storage::new(borrowed_storage_config).await?;
    let filename = get_filename(source_config);
    let path = storage.write(&filename, bytes).await?;
    Ok(path)
}

pub async fn restore<SO, ST>(
    source_config: SO,
    storage_config: ST,
    filename: &str,
    drop_database: bool,
) -> Result<()>
where
    SO: Borrow<SourceConfig>,
    ST: Borrow<StorageConfig>,
{
    let borrowed_source_config = source_config.borrow();
    let borrowed_storage_config = storage_config.borrow();

    let storage = Storage::new(borrowed_storage_config).await?;
    let bytes = storage.read(filename).await?;
    restore_source(borrowed_source_config, bytes, drop_database).await?;
    Ok(())
}

pub async fn list<ST>(storage_config: ST) -> Result<Vec<Entry>>
where
    ST: Borrow<StorageConfig>,
{
    let borrowed_storage_config = storage_config.borrow();
    let storage = Storage::new(borrowed_storage_config).await?;
    let entries = storage.list().await?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{databases::configs::PGSourceConfig, storage::configs::S3StorageConfig};
    use dotenv::dotenv;

    use super::*;

    #[tokio::test]
    async fn test_backup() {
        dotenv().ok();

        let storage_config = StorageConfig::S3(S3StorageConfig {
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string()),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            name: env::var("S3_CONFIG_NAME").unwrap_or_else(|_| "s3-test".to_string()),
            prefix: Some(env::var("S3_PREFIX").unwrap_or_default()),
        });

        let source_config = SourceConfig::PG(PGSourceConfig {
            name: "test".into(),
            database: "api".into(),
            host: "localhost".into(),
            port: 5432,
            username: "postgres".into(),
            password: Some("postgres".into()),
        });

        let dump_path = backup(&source_config, &storage_config)
            .await
            .expect("Failed to backup");

        let entries = list(&storage_config).await.expect("Failed to list");
        let entry = entries.iter().find(|entry| entry.path() == dump_path);

        assert!(entry.is_some());
    }

    #[tokio::test]
    async fn test_restore() {
        dotenv().ok();

        let storage_config = StorageConfig::S3(S3StorageConfig {
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string()),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            name: env::var("S3_CONFIG_NAME").unwrap_or_else(|_| "s3-test".to_string()),
            prefix: Some(env::var("S3_PREFIX").unwrap_or_default()),
        });

        let source_config = SourceConfig::PG(PGSourceConfig {
            name: "test".into(),
            database: "api".into(),
            host: "localhost".into(),
            port: 5432,
            username: "postgres".into(),
            password: Some("Gwt2tmrGtN4OZ3oL577E".into()),
        });

        let filename = "test-api-2025-03-15-100759-1853fe65.gz";

        restore(&source_config, &storage_config, &filename, true)
            .await
            .expect("Unable to restore")
    }

    #[tokio::test]
    async fn test_e2e_backup_restore() {
        dotenv().ok();

        let storage_config = StorageConfig::S3(S3StorageConfig {
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string()),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            name: env::var("S3_CONFIG_NAME").unwrap_or_else(|_| "s3-test".to_string()),
            prefix: Some(env::var("S3_PREFIX").unwrap_or_default()),
        });

        let source_config = SourceConfig::PG(PGSourceConfig {
            name: "test".into(),
            database: "api".into(),
            host: "localhost".into(),
            port: 5432,
            username: "postgres".into(),
            password: Some("postgres".into()),
        });

        let dump_path = backup(&source_config, &storage_config)
            .await
            .expect("Failed to backup");

        let entries = list(&storage_config).await.expect("Failed to list");
        let entry = entries.iter().find(|entry| entry.path() == dump_path);

        assert!(entry.is_some());

        let storage = Storage::new(&storage_config)
            .await
            .expect("Unable to create storage");

        let filename = storage.get_filename_from_path(&dump_path);

        restore(&source_config, &storage_config, &filename, true)
            .await
            .expect("Unable to restore")
    }
}
