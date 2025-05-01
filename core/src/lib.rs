use std::{
    borrow::Borrow,
    io::{Cursor, Read},
};

use anyhow::Result;

pub mod common;
pub mod databases;
pub mod folders;
pub mod storage;
mod tests;
pub mod tunnel;

use bytes::Bytes;
use chrono::Utc;
pub use common::get_backup_key;
use common::{get_filename, get_source_name};
use databases::configs::SourceConfig;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use opendal::Entry;
use storage::{configs::StorageConfig, storage::Storage};
use tar::{Archive, Builder, Header};

pub async fn is_connected<SO>(source_config: SO) -> Result<bool>
where
    SO: Borrow<SourceConfig>,
{
    let is_connected = databases::is_connected(source_config.borrow()).await?;
    Ok(is_connected)
}

pub async fn backup<SO, ST>(source_config: SO, storage_config: ST) -> Result<String>
where
    SO: Borrow<SourceConfig>,
    ST: Borrow<StorageConfig>,
{
    let borrowed_source_config = source_config.borrow();
    let borrowed_storage_config = storage_config.borrow();

    let bytes = databases::backup(borrowed_source_config).await?;

    // Compress bytes
    let source_name = get_source_name(borrowed_source_config);
    let filename = get_filename(borrowed_source_config);

    let internal_filename = format!("backup_{}.sql", source_name);

    let cursor = Cursor::new(Vec::new());
    let encoder = GzEncoder::new(cursor, Compression::default());
    let mut builder = Builder::new(encoder);

    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(Utc::now().timestamp() as u64);

    builder.append_data(&mut header, &internal_filename, Cursor::new(bytes.to_vec()))?;

    let encoder = builder.into_inner()?;
    let compressed_data = encoder.finish()?;
    let compressed_bytes = Bytes::from(compressed_data.into_inner());

    // Write compressed bytes to storage
    let storage = Storage::new(borrowed_storage_config).await?;
    let path = storage.write(&filename, compressed_bytes).await?;

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

    let cursor = std::io::Cursor::new(bytes);
    let gz_decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz_decoder);

    let mut extracted_content = Vec::new();
    let mut found_file = false;

    for entry in archive.entries()? {
        let mut file = entry?;

        if file.header().entry_type().is_dir() {
            continue;
        }

        let path = file.path()?;
        let extension = match path.extension() {
            Some(extension) => extension.to_str().unwrap_or(""),
            None => "",
        };

        if extension == "sql" {
            file.read_to_end(&mut extracted_content)?;
            found_file = true;

            break;
        }
    }

    if !found_file {
        return Err(anyhow::anyhow!("No valid backup file found in the archive"));
    }

    let decompressed_bytes = Bytes::from(extracted_content);

    databases::restore(borrowed_source_config, decompressed_bytes, drop_database).await?;

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

pub async fn cleanup<ST>(
    storage_config: ST,
    retention_days: u64,
    dry_run: bool,
) -> Result<(usize, u64)>
where
    ST: Borrow<StorageConfig>,
{
    let borrowed_storage_config = storage_config.borrow();
    let storage = Storage::new(borrowed_storage_config).await?;
    let result = storage.cleanup(retention_days, dry_run).await?;
    Ok(result)
}
