use log::info;
use serde::{Deserialize, Serialize};
use vprs3bkp_core::{
    download_backup, get_backup_key,
    postgres::{self, is_postgres_connected_default_timeout},
    restore_postgres, upload_to_s3, BackupInfo,
};

use crate::utils::get_s3_client;

// Define types that match your TypeScript interfaces
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct S3StorageProvider {
    #[serde(rename = "type")]
    pub provider_type: String,
    pub region: Option<String>,
    pub bucket: String,
    pub endpoint: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BackupListItem {
    name: String,
    size: u64,
    timestamp: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresBackupSource {
    #[serde(rename = "type")]
    backup_source_type: String,
    database_type: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConnectionResult {
    connected: bool,
}

#[tauri::command]
pub async fn list_backups(storage_provider: S3StorageProvider) -> Result<Vec<BackupInfo>, String> {
    // Ensure we're dealing with an S3 provider
    if storage_provider.provider_type != "s3" {
        return Err("Only S3 storage providers are supported".to_string());
    }

    let s3_client = get_s3_client(&storage_provider).await?;

    let backups = vprs3bkp_core::list_backups(
        &s3_client,
        &storage_provider.bucket.trim(),
        "",
        None,
        None,
        100,
    )
    .await
    .map_err(|e| format!("Failed to list objects: {}", e))?;

    Ok(backups)
}

#[tauri::command]
pub async fn backup_source(
    backup_source: PostgresBackupSource,
    storage_provider: S3StorageProvider,
) -> Result<String, String> {
    if storage_provider.provider_type != "s3" {
        return Err("Only S3 storage providers are supported".to_string());
    }

    let s3_client = get_s3_client(&storage_provider).await?;

    let backup_bytes = postgres::backup_postgres_with_options(
        &backup_source.database,
        &backup_source.host,
        backup_source.port,
        &backup_source.username,
        Some(backup_source.password.as_str()),
        9,
        false,
    )
    .await
    .map_err(|e| format!("Failed to backup postgres: {}", e))?;

    let key = get_backup_key("", "postgres", &backup_source.database);
    upload_to_s3(
        &s3_client,
        &storage_provider.bucket,
        &key,
        aws_sdk_s3::primitives::ByteStream::from(backup_bytes),
    )
    .await
    .map_err(|e| format!("Failed to upload backup: {}", e))?;

    Ok("ok".into())
}

#[tauri::command]
pub async fn restore_backup(
    backup_key: String,
    backup_source: PostgresBackupSource,
    storage_provider: S3StorageProvider,
) -> Result<String, String> {
    if storage_provider.provider_type != "s3" {
        return Err("Only S3 storage providers are supported".to_string());
    }

    let s3_client = get_s3_client(&storage_provider).await?;

    // Download the backup
    let backup_data = download_backup(&s3_client, &storage_provider.bucket, &backup_key)
        .await
        .map_err(|e| format!("Failed to download backup: {}", e))?;

    // Restore the database
    restore_postgres(
        &backup_source.database,
        &backup_source.host,
        backup_source.port,
        &backup_source.username,
        Some(&backup_source.password),
        backup_data,
        true,
        false,
    )
    .await
    .map_err(|e| format!("Failed to restore backup: {}", e))?;

    info!("PostgreSQL database restore completed successfully");

    Ok("ok".into())
}

#[tauri::command]
pub async fn verify_backup_source_connection(
    backup_source: PostgresBackupSource,
) -> Result<BackupConnectionResult, String> {
    let connected = is_postgres_connected_default_timeout(
        &backup_source.host,
        backup_source.port,
        &backup_source.database,
        &backup_source.username,
        Some(&backup_source.password),
    )
    .await
    .map_err(|e| format!("Failed to check backup source connection: {}", e))?;

    Ok(BackupConnectionResult { connected })
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
