use std::time::Duration;

use crate::databases::{
    connection::DatabaseConfig, BackupOptions, DatabaseMetadata, RestoreOptions,
    SQLDatabaseConnection,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Pool, Postgres,
};

pub struct PostgreSQLConnection {
    config: DatabaseConfig,
    pool: Pool<Postgres>,
}

impl PostgreSQLConnection {
    async fn new(config: DatabaseConfig) -> Result<Self> {
        let mut connect_options = PgConnectOptions::new()
            .host(&config.host)
            .username(&config.username)
            .database(&config.database)
            .port(config.port);

        connect_options = match &config.password {
            Some(password) => connect_options.password(&password),
            None => connect_options,
        };

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(connect_options)
            .await?;

        Ok(Self { config, pool })
    }
}

#[async_trait]
impl SQLDatabaseConnection for PostgreSQLConnection {
    async fn get_metadata(&self) -> Result<DatabaseMetadata> {
        let version: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to get database version: {}", e))?;

        Ok(DatabaseMetadata { version: version.0 })
    }

    async fn test(&self) -> Result<bool> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| true)
            .map_err(|e| anyhow!("Connection test failed: {}", e))
    }

    async fn backup(&self, backup_options: BackupOptions) -> Result<()> {
        Ok(())
    }

    async fn restore(&self, restore_options: RestoreOptions) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod postgresql_connection_test {
    use super::*;
    use crate::databases::connection::ConnectionType;
    use dotenv::dotenv;
    use std::env;

    async fn get_connection() -> Result<PostgreSQLConnection> {
        dotenv().ok();

        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("POSTGRESQL_PASSWORD").unwrap_or_default();
        let connection = PostgreSQLConnection::new(DatabaseConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: ConnectionType::PostgreSQL,
            host: env::var("POSTGRESQL_HOST").unwrap_or_default(),
            password: Some(password),
            username: env::var("POSTGRESQL_USERNAME").unwrap_or_default(),
            database: env::var("POSTGRESQL_NAME").unwrap_or_default(),
            port,
            ssh_tunnel: None,
        })
        .await?;

        Ok(connection)
    }

    #[tokio::test]
    async fn test_01_connection_test() {
        let connection = get_connection().await.expect("Failed to get connection");
        let is_connected = connection.test().await.expect("Failed to test connection");
        assert!(is_connected)
    }

    #[tokio::test]
    async fn test_02_get_metadata() {
        let connection = get_connection().await.expect("Failed to get connection");
        let metadata = connection
            .get_metadata()
            .await
            .expect("Failed to get metadata");
        assert!(metadata.version.contains("15.12"));
    }
}
