use anyhow::{anyhow, Context, Result};
use chrono::DateTime;
use opendal::{
    layers::LoggingLayer,
    services::{Fs, S3},
    BufferStream, Entry, Operator,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    sync::Arc,
};
use tokio::sync::Mutex;

use crate::common::extract_timestamp_from_filename;

use super::io::{StorageReader, StorageWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageCredentials {
    None,
    Basic {
        username: String,
        password: String,
    },
    AccessKey {
        access_key: String,
        secret_key: String,
    },
    PrivateKey {
        username: String,
        key_path: String,
        passphrase: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    FileSystem,
    S3,
    // WebDAV,
    // SFTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub id: String,
    pub name: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3StorageConfig {
    pub id: String,
    pub name: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageConfig {
    Local(LocalStorageConfig),
    S3(S3StorageConfig),
}

#[derive(Clone)]
pub struct StorageProvider {
    pub config: StorageConfig,
    pub operator: Operator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOptions {
    pub latest_only: Option<bool>,
    pub limit: Option<usize>,
}

impl StorageProvider {
    pub fn new(config: StorageConfig) -> anyhow::Result<Self> {
        let operator = match &config {
            StorageConfig::Local(config) => {
                let builder = Fs::default().root(&config.location);
                Operator::new(builder)?
                    .layer(LoggingLayer::default())
                    .finish()
            }
            StorageConfig::S3(config) => {
                let mut builder = S3::default()
                    .root(&config.location)
                    .bucket(&config.bucket)
                    .region(&config.region)
                    .access_key_id(&config.access_key)
                    .secret_access_key(&config.secret_key);

                builder = match &config.endpoint {
                    Some(endpoint) => builder.endpoint(endpoint),
                    None => builder,
                };

                Operator::new(builder)?
                    .layer(LoggingLayer::default())
                    .finish()
            }
        };

        Ok(StorageProvider { config, operator })
    }

    pub async fn test(&self) -> Result<bool> {
        self.operator
            .list_with("/")
            .recursive(true)
            .limit(1)
            .await?;

        Ok(true)
    }

    pub async fn list(&self, options: ListOptions) -> Result<Vec<Entry>> {
        let limit = options.limit.unwrap_or(1000);
        let latest_only = options.latest_only.unwrap_or(false);

        let result = self
            .operator
            .list_with("")
            .recursive(true)
            .limit(limit)
            .await
            .context(format!("Failed to list backups"))?;

        let mut filtered_results: Vec<Entry> = result
            .into_iter()
            .filter(|entry| entry.metadata().is_file())
            .collect();

        filtered_results.sort_by(|a, b| {
            let a_timestamp =
                extract_timestamp_from_filename(a.name()).unwrap_or(DateTime::default());

            let b_timestamp =
                extract_timestamp_from_filename(b.name()).unwrap_or(DateTime::default());

            b_timestamp.cmp(&a_timestamp)
        });

        if latest_only {
            match filtered_results.first() {
                Some(entry) => return Ok(vec![entry.clone()]),
                None => return Err(anyhow!("No entry found")),
            }
        }

        Ok(filtered_results)
    }

    pub async fn create_writer(&self, filename: &str) -> Result<Box<dyn Write + Send + Unpin>> {
        let op_writer = self.operator.writer(filename).await?;
        Ok(Box::new(StorageWriter::new(op_writer)))
    }

    pub async fn create_stream(&self, filename: &str) -> Result<Arc<Mutex<BufferStream>>> {
        let metadata = self.operator.stat(filename).await?;
        let file_size = metadata.content_length() as usize;
        let chunk_size = if file_size > 512 { 512 } else { file_size };

        let stream = self
            .operator
            .reader_with(filename)
            .chunk(chunk_size as usize)
            .await?
            .into_stream(0u64..(file_size as u64))
            .await?;

        Ok(Arc::new(Mutex::new(stream)))
    }

    pub async fn create_reader(&self, filename: &str) -> Result<Box<dyn Read + Send + Unpin>> {
        Ok(Box::new(StorageReader::new(
            self.operator.clone(),
            filename.to_string(),
        )))
    }
}
