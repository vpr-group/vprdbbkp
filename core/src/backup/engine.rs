use crate::{databases::connection::DatabaseConnection, storage::provider::StorageProvider};
use anyhow::Result;

pub struct BackupEngine {
    database_connection: DatabaseConnection,
    storage_provider: StorageProvider,
}

impl BackupEngine {
    fn new(database_connection: DatabaseConnection, storage_provider: StorageProvider) -> Self {
        Self {
            database_connection,
            storage_provider,
        }
    }

    pub async fn backup(&self) -> Result<()> {
        let mut writer = self.storage_provider.create_writer("test-bkp").await?;

        self.database_connection
            .connection
            .backup(&mut writer)
            .await?;

        writer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod backup_engine_tests {

    use std::env;

    use anyhow::Result;
    use dotenv::dotenv;

    use tempfile::tempdir;

    use crate::{
        backup::engine::BackupEngine,
        databases::connection::{ConnectionType, DatabaseConfig, DatabaseConnection},
        storage::provider::{LocalStorageConfig, S3StorageConfig, StorageConfig, StorageProvider},
    };

    fn get_local_provider() -> Result<StorageProvider> {
        dotenv().ok();

        let temp_path = tempdir()?;
        let config = StorageConfig::Local(LocalStorageConfig {
            id: "test".into(),
            name: "local".into(),
            location: temp_path.path().to_str().unwrap().to_string(),
        });
        let provider = StorageProvider::new(config)?;
        Ok(provider)
    }

    fn get_s3_provider() -> Result<StorageProvider> {
        dotenv().ok();

        let endpoint = env::var("S3_ENDPOINT")
            .unwrap_or_else(|_| "https://s3.pub1.infomaniak.cloud/".to_string());

        let config = StorageConfig::S3(S3StorageConfig {
            id: "test".into(),
            name: "s3".into(),
            access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "test-bkp".to_string()),
            endpoint: Some(endpoint),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            location: "/s3-test".into(),
        });

        let provider = StorageProvider::new(config)?;

        Ok(provider)
    }

    async fn get_postgresql_connection() -> Result<DatabaseConnection> {
        dotenv().ok();

        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("POSTGRESQL_PASSWORD").unwrap_or_default();

        let config = DatabaseConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: ConnectionType::PostgreSQL,
            host: env::var("POSTGRESQL_HOST").unwrap_or_default(),
            password: Some(password),
            username: env::var("POSTGRESQL_USERNAME").unwrap_or_default(),
            database: env::var("POSTGRESQL_NAME").unwrap_or_default(),
            port,
            ssh_tunnel: None,
        };

        let connection = DatabaseConnection::new(config).await?;

        Ok(connection)
    }

    #[tokio::test]
    async fn test_01_backup_postgresql() {
        let storage_provider = get_s3_provider().expect("Failed to get local storage provider");
        let db_connection = get_postgresql_connection()
            .await
            .expect("Failed to get postgresql connection");

        let engine = BackupEngine::new(db_connection, storage_provider);

        engine.backup().await.expect("Failed to backup");
    }
}
