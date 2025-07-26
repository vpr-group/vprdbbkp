use dbkp_core::{
    databases::{DatabaseConfig, DatabaseConnection},
    storage::{
        provider::{StorageConfig, StorageProvider},
        Entry,
    },
    DbBkp, RestoreOptions,
};
use serde::{Deserialize, Serialize};
// use dbkp_core::{
//     databases::{configs::databaseConfig, is_connected},
//     storage::configs::StorageConfig,
// };

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConnectionResult {
    connected: bool,
}

#[tauri::command]
pub async fn list(storage_config: StorageConfig) -> Result<Vec<Entry>, String> {
    let storage_provider = StorageProvider::new(storage_config)
        .map_err(|e| format!("Failed to create storage provider: {}", e))?;

    let entries = storage_provider
        .list()
        .await
        .map_err(|e| format!("Failed to list entries: {}", e))?;

    Ok(entries)
}

#[tauri::command]
pub async fn backup(
    database_config: DatabaseConfig,
    storage_config: StorageConfig,
) -> Result<String, String> {
    let database_connection = DatabaseConnection::new(database_config)
        .await
        .map_err(|e| format!("Failed to create database connection: {}", e))?;

    let storage_provider = StorageProvider::new(storage_config)
        .map_err(|e| format!("Failed to create storage provider: {}", e))?;

    let db_bkp = DbBkp::new(database_connection, storage_provider);

    db_bkp
        .backup()
        .await
        .map_err(|e| format!("Failed to backup database: {}", e))?;

    Ok("ok".into())
}

// #[tauri::command]
// pub async fn restore(
//     filename: String,
//     source_config: databaseConfig,
//     storage_config: StorageConfig,
//     drop_database: bool,
// ) -> Result<String, String> {
//     dbkp_core::restore(source_config, storage_config, &filename, drop_database)
//         .await
//         .map_err(|e| format!("Failed to restore backup: {}", e))?;

//     info!("PostgreSQL database restore completed successfully");

//     Ok("ok".into())
// }

#[tauri::command]
pub async fn restore(
    filename: String,
    database_config: DatabaseConfig,
    storage_config: StorageConfig,
    drop_database: bool,
) -> Result<String, String> {
    let database_connection = DatabaseConnection::new(database_config)
        .await
        .map_err(|e| format!("Failed to create database connection: {}", e))?;

    let storage_provider = StorageProvider::new(storage_config)
        .map_err(|e| format!("Failed to create storage provider: {}", e))?;

    let db_bkp = DbBkp::new(database_connection, storage_provider);

    db_bkp
        .restore(RestoreOptions {
            name: filename,
            compression_format: None,
            drop_database_first: Some(drop_database),
        })
        .await
        .map_err(|e| format!("Failed to restore backup: {}", e))?;

    Ok("ok".into())
}

#[tauri::command]
pub async fn test_connection(
    database_config: DatabaseConfig,
) -> Result<BackupConnectionResult, String> {
    let database_connection = DatabaseConnection::new(database_config)
        .await
        .map_err(|e| format!("Failed to create database connection: {}", e))?;

    let connected = database_connection
        .connection
        .test()
        .await
        .map_err(|e| format!("Failed to test database connection: {}", e))?;

    Ok(BackupConnectionResult { connected })
}
