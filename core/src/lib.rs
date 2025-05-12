use anyhow::Result;
use common::get_default_backup_name;
use compression::{CompressionFormat, Compressor, Decompressor};
use databases::DatabaseConnection;
use flate2::Compression;
use opendal::Entry;
use serde::{Deserialize, Serialize};
use storage::provider::StorageProvider;

pub mod common;
pub mod compression;
pub mod databases;
pub mod folders;
pub mod storage;
mod tests;

#[derive(Clone, Serialize, Deserialize)]
pub struct BackupOptions {
    name: Option<String>,
    compression_format: Option<CompressionFormat>,
    compression_level: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    name: String,
    compression_format: Option<CompressionFormat>,
}

pub struct DbBkp {
    database_connection: DatabaseConnection,
    storage_provider: StorageProvider,
}

impl DbBkp {
    pub fn new(database_connection: DatabaseConnection, storage_provider: StorageProvider) -> Self {
        Self {
            database_connection,
            storage_provider,
        }
    }

    pub async fn backup_with(&self, options: Option<BackupOptions>) -> Result<String> {
        let options = match options {
            Some(options) => options,
            None => BackupOptions {
                name: None,
                compression_format: None,
                compression_level: None,
            },
        };

        let compression_format = options
            .compression_format
            .unwrap_or(CompressionFormat::Gzip);
        let compression_level = options.compression_level.unwrap_or(9);
        let name = match options.name {
            Some(name) => name,
            None => get_default_backup_name(&self.database_connection.config, &compression_format),
        };

        let writer = self.storage_provider.create_writer(&name).await?;
        let mut compressed_writed = Compressor::new(
            writer,
            compression_format,
            Compression::new(compression_level),
        );

        self.database_connection
            .connection
            .backup(&mut compressed_writed)
            .await?;

        let mut writer = compressed_writed.finish()?;
        writer.flush()?;

        Ok(name)
    }

    pub async fn backup(&self) -> Result<String> {
        self.backup_with(None).await
    }

    pub async fn restore(&self, options: RestoreOptions) -> Result<()> {
        let compression_format = options
            .compression_format
            .unwrap_or(CompressionFormat::Gzip);

        let reader = self.storage_provider.create_reader(&options.name).await?;
        let mut compressed_reader = Decompressor::new(reader, compression_format);

        self.database_connection
            .connection
            .restore(&mut compressed_reader)
            .await?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Entry>> {
        let entries = self.storage_provider.list().await?;
        Ok(entries)
    }
}
