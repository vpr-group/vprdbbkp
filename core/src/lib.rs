use std::io::Write;

use anyhow::{anyhow, Result};
use common::get_default_backup_name;
use compression::{CompressionFormat, Compressor, Decompressor};
use databases::DatabaseConnection;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use storage::provider::{ListOptions, StorageProvider};

use crate::storage::Entry;

pub mod archives;
pub mod common;
pub mod compression;
pub mod databases;
pub mod folders;
pub mod storage;
mod test_utils;
mod tests;

#[derive(Clone, Serialize, Deserialize)]
pub struct BackupOptions {
    name: Option<String>,
    compression_format: Option<CompressionFormat>,
    compression_level: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    pub name: String,
    pub compression_format: Option<CompressionFormat>,
    pub drop_database_first: Option<bool>,
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

    pub async fn test(&self) -> Result<bool> {
        let is_database_connected = self.database_connection.connection.test().await?;
        let is_storage_connected = self.storage_provider.test().await?;

        if !is_database_connected {
            return Err(anyhow!("Failed to connect to the database"));
        } else if !is_storage_connected {
            return Err(anyhow!("Failed to connect to the storage provider"));
        }

        return Ok(true);
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
            .restore_with_options(
                &mut compressed_reader,
                databases::RestoreOptions {
                    drop_database_first: match options.drop_database_first {
                        Some(drop) => drop,
                        None => false,
                    },
                },
            )
            .await?;

        Ok(())
    }

    pub async fn list_with_options(&self, options: ListOptions) -> Result<Vec<Entry>> {
        let entries = self.storage_provider.list_with_options(options).await?;
        Ok(entries)
    }

    pub async fn list(&self) -> Result<Vec<Entry>> {
        let entries = self.storage_provider.list().await?;

        Ok(entries)
    }
}
