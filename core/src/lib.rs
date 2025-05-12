use anyhow::Result;
use compression::{CompressionFormat, Compressor, Decompressor};
use databases::DatabaseConnection;
use flate2::Compression;
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
    name: String,
    compression_format: CompressionFormat,
    compression_level: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    name: String,
    compression_format: CompressionFormat,
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

    pub async fn backup(&self, options: BackupOptions) -> Result<()> {
        let writer = self.storage_provider.create_writer(&options.name).await?;
        let mut compressed_writed = Compressor::new(
            writer,
            options.compression_format,
            Compression::new(options.compression_level),
        );

        self.database_connection
            .connection
            .backup(&mut compressed_writed)
            .await?;

        let mut writer = compressed_writed.finish()?;
        writer.flush()?;

        Ok(())
    }

    pub async fn restore(&self, options: RestoreOptions) -> Result<()> {
        let reader = self.storage_provider.create_reader(&options.name).await?;
        let mut compressed_reader = Decompressor::new(reader, options.compression_format);

        self.database_connection
            .connection
            .restore(&mut compressed_reader)
            .await?;

        Ok(())
    }
}
