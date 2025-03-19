use crate::utils::serializable_entry::{entries_to_serializable, SerializableEntry};
use log::info;
use serde::{Deserialize, Serialize};
use vprs3bkp_core::{
    databases::{configs::SourceConfig, is_database_connected},
    storage::configs::StorageConfig,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConnectionResult {
    connected: bool,
}

#[tauri::command]
pub async fn list(storage_config: StorageConfig) -> Result<Vec<SerializableEntry>, String> {
    let entries = vprs3bkp_core::list(storage_config)
        .await
        .map_err(|e| format!("Failed to list objects: {}", e))?;

    Ok(entries_to_serializable(entries))
}

#[tauri::command]
pub async fn backup(
    source_config: SourceConfig,
    storage_config: StorageConfig,
) -> Result<String, String> {
    vprs3bkp_core::backup(source_config, storage_config)
        .await
        .map_err(|e| format!("Failed to backup: {}", e))?;
    Ok("ok".into())
}

#[tauri::command]
pub async fn restore(
    filename: String,
    source_config: SourceConfig,
    storage_config: StorageConfig,
    drop_database: bool,
) -> Result<String, String> {
    vprs3bkp_core::restore(source_config, storage_config, &filename, drop_database)
        .await
        .map_err(|e| format!("Failed to restore backup: {}", e))?;

    info!("PostgreSQL database restore completed successfully");

    Ok("ok".into())
}

#[tauri::command]
pub async fn verify_connection(
    source_config: SourceConfig,
) -> Result<BackupConnectionResult, String> {
    let connected = is_database_connected(source_config)
        .await
        .map_err(|e| format!("Failed to check source connection: {}", e))?;

    Ok(BackupConnectionResult { connected })
}
