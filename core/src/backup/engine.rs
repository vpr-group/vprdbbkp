use crate::{databases::DatabaseConnection, storage::provider::StorageProvider};
use anyhow::Result;

pub struct BackupEngine {
    database_connection: DatabaseConnection,
    storage_provider: StorageProvider,
}

impl BackupEngine {
    pub fn new(database_connection: DatabaseConnection, storage_provider: StorageProvider) -> Self {
        Self {
            database_connection,
            storage_provider,
        }
    }

    pub async fn backup(&self, backup_name: &str) -> Result<()> {
        let mut writer = self.storage_provider.create_writer(backup_name).await?;

        self.database_connection
            .connection
            .backup(&mut writer)
            .await?;

        writer.flush()?;

        Ok(())
    }

    pub async fn restore(&self, backup_name: &str) -> Result<()> {
        let mut reader = self.storage_provider.create_reader(backup_name).await?;

        self.database_connection
            .connection
            .restore(&mut reader)
            .await?;

        Ok(())
    }
}
