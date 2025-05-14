#[cfg(test)]
mod vprdbbkp_tests {

    use std::env;

    use anyhow::Result;
    use dotenv::dotenv;

    use tempfile::tempdir;

    use crate::{
        databases::{
            mysql::connection::MySqlConnection,
            postgres::connection::PostgreSqlConnection,
            ssh_tunnel::{SshAuthMethod, SshTunnelConfig},
            ConnectionType, DatabaseConfig, DatabaseConnection,
        },
        storage::provider::{LocalStorageConfig, S3StorageConfig, StorageConfig, StorageProvider},
        DbBkp, RestoreOptions,
    };

    fn initialize() {
        dotenv().ok();
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
    }

    fn get_local_provider() -> Result<StorageProvider> {
        initialize();

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
        initialize();

        let location = format!("s3_provider_test_{}", chrono::Utc::now().timestamp());

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
            location,
        });

        let provider = StorageProvider::new(config)?;

        Ok(provider)
    }

    fn get_postgresql_config() -> Result<DatabaseConfig> {
        initialize();

        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("POSTGRESQL_PASSWORD").unwrap_or_default();

        let config = DatabaseConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: ConnectionType::PostgreSql,
            host: env::var("POSTGRESQL_HOST").unwrap_or_default(),
            password: Some(password),
            username: env::var("POSTGRESQL_USERNAME").unwrap_or_default(),
            database: env::var("POSTGRESQL_NAME").unwrap_or_default(),
            port,
            ssh_tunnel: None,
        };

        Ok(config)
    }

    fn get_mysql_config() -> Result<DatabaseConfig> {
        initialize();

        let port: u16 = env::var("MYSQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("MYSQL_PASSWORD").unwrap_or_default();

        let config = DatabaseConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: ConnectionType::MySql,
            host: env::var("MYSQL_HOST").unwrap_or_default(),
            password: Some(password),
            username: env::var("MYSQL_USERNAME").unwrap_or_default(),
            database: env::var("MYSQL_NAME").unwrap_or_default(),
            port,
            ssh_tunnel: None,
        };

        Ok(config)
    }

    async fn get_postgresql_tunneled_config() -> Result<DatabaseConfig> {
        initialize();

        let port: u16 = env::var("DB_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("DB_PASSWORD").unwrap_or_default();

        let mut config = get_postgresql_config()?;

        config.password = Some(password);
        config.port = port;
        config.username = env::var("DB_USERNAME").unwrap_or_default();
        config.database = env::var("DB_NAME").unwrap_or_default();

        config.ssh_tunnel = Some(SshTunnelConfig {
            host: env::var("SSH_HOST").unwrap_or_default(),
            username: env::var("SSH_USERNAME").unwrap_or_default(),
            port: 22,
            auth_method: SshAuthMethod::PrivateKey {
                key_path: env::var("SSH_KEY_PATH").unwrap_or_default(),
                passphrase_key: None,
            },
        });

        Ok(config)
    }

    #[tokio::test]
    async fn test_01_postgresql_backup() {
        initialize();
        let config = get_postgresql_config().expect("Failed to get postgresql config");

        let postgresql_connection = PostgreSqlConnection::new(config.clone())
            .await
            .expect("Failed to get postgresql connection");

        let database_connection = DatabaseConnection::new(config.clone())
            .await
            .expect("Failed to get database connection");

        let storage_provider = get_local_provider().expect("Failed to get local storage provider");

        let engine = DbBkp::new(database_connection, storage_provider);

        sqlx::query("DROP TABLE IF EXISTS backup_test_table")
            .execute(&postgresql_connection.pool)
            .await
            .expect("Failed to drop test table");

        sqlx::query(
            "CREATE TABLE backup_test_table (id SERIAL PRIMARY KEY, name TEXT, value INTEGER)",
        )
        .execute(&postgresql_connection.pool)
        .await
        .expect("Failed to create test table");

        sqlx::query("INSERT INTO backup_test_table (name, value) VALUES ('test1', 100), ('test2', 200), ('test3', 300)")
        .execute(&postgresql_connection.pool)
        .await
        .expect("Failed to insert test data");

        let rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
                .fetch_all(&postgresql_connection.pool)
                .await
                .expect("Failed to fetch test data");

        assert_eq!(rows.len(), 3, "Should have 3 rows before backup");

        let backup_name = engine.backup().await.expect("Failed to backup");

        sqlx::query("UPDATE backup_test_table SET value = 999 WHERE name = 'test1'")
            .execute(&postgresql_connection.pool)
            .await
            .expect("Failed to update test data");

        sqlx::query("DELETE FROM backup_test_table WHERE name = 'test3'")
            .execute(&postgresql_connection.pool)
            .await
            .expect("Failed to delete test data");

        let modified_rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
                .fetch_all(&postgresql_connection.pool)
                .await
                .expect("Failed to fetch modified data");

        assert_eq!(modified_rows.len(), 2, "Should have 2 rows after deletion");
        assert_eq!(modified_rows[0].1, 999, "Value should be modified");

        engine
            .restore(RestoreOptions {
                name: backup_name,
                compression_format: None,
            })
            .await
            .expect("Failed to restore");

        let postgresql_connection = PostgreSqlConnection::new(config.clone())
            .await
            .expect("Failed to get postgresql connection");

        let restored_rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
                .fetch_all(&postgresql_connection.pool)
                .await
                .expect("Failed to fetch restored data");

        assert_eq!(restored_rows.len(), 3, "Should have 3 rows after restore");

        let test1_row = restored_rows
            .iter()
            .find(|(name, _)| name == "test1")
            .expect("Should have test1 row after restore");

        assert_eq!(
            test1_row.1, 100,
            "test1 value should be restored to original"
        );

        let test3_exists = restored_rows.iter().any(|(name, _)| name == "test3");
        assert!(test3_exists, "test3 should be restored");
    }

    #[ignore]
    #[tokio::test]
    async fn test_02_postgresql_tunneled_backup() {
        initialize();
        let config = get_postgresql_tunneled_config()
            .await
            .expect("Failed to get postgresql connection config");

        let database_connection = DatabaseConnection::new(config.clone())
            .await
            .expect("Failed to get database connection");

        let storage_provider = get_s3_provider().expect("Failed to get local storage provider");

        let engine = DbBkp::new(database_connection, storage_provider);

        let backup_name = engine.backup().await.expect("Failed to backup");
        let entries = engine.list().await.expect("Failed to list backups");
        let entry = entries.iter().find(|e| e.name() == backup_name);

        assert!(entry.is_some());
    }

    #[tokio::test]
    async fn test_03_mysql_backup() {
        initialize();
        let config = get_mysql_config().expect("Failed to get mysql config");

        let mysql_connection = MySqlConnection::new(config.clone())
            .await
            .expect("Failed to get mysql connection");

        let database_connection = DatabaseConnection::new(config.clone())
            .await
            .expect("Failed to get database connection");

        let storage_provider = get_local_provider().expect("Failed to get local storage provider");

        let engine = DbBkp::new(database_connection, storage_provider);

        sqlx::query("DROP TABLE IF EXISTS backup_test_table")
            .execute(&mysql_connection.pool)
            .await
            .expect("Failed to drop test table");

        sqlx::query(
            "CREATE TABLE backup_test_table (id SERIAL PRIMARY KEY, name TEXT, value INTEGER)",
        )
        .execute(&mysql_connection.pool)
        .await
        .expect("Failed to create test table");

        sqlx::query("INSERT INTO backup_test_table (name, value) VALUES ('test1', 100), ('test2', 200), ('test3', 300)")
        .execute(&mysql_connection.pool)
        .await
        .expect("Failed to insert test data");

        let rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
                .fetch_all(&mysql_connection.pool)
                .await
                .expect("Failed to fetch test data");

        assert_eq!(rows.len(), 3, "Should have 3 rows before backup");

        let backup_name = engine.backup().await.expect("Failed to backup");

        sqlx::query("UPDATE backup_test_table SET value = 999 WHERE name = 'test1'")
            .execute(&mysql_connection.pool)
            .await
            .expect("Failed to update test data");

        sqlx::query("DELETE FROM backup_test_table WHERE name = 'test3'")
            .execute(&mysql_connection.pool)
            .await
            .expect("Failed to delete test data");

        let modified_rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
                .fetch_all(&mysql_connection.pool)
                .await
                .expect("Failed to fetch modified data");

        assert_eq!(modified_rows.len(), 2, "Should have 2 rows after deletion");
        assert_eq!(modified_rows[0].1, 999, "Value should be modified");

        engine
            .restore(RestoreOptions {
                name: backup_name,
                compression_format: None,
            })
            .await
            .expect("Failed to restore");

        let postgresql_connection = MySqlConnection::new(config.clone())
            .await
            .expect("Failed to get postgresql connection");

        let restored_rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
                .fetch_all(&postgresql_connection.pool)
                .await
                .expect("Failed to fetch restored data");

        assert_eq!(restored_rows.len(), 3, "Should have 3 rows after restore");

        let test1_row = restored_rows
            .iter()
            .find(|(name, _)| name == "test1")
            .expect("Should have test1 row after restore");

        assert_eq!(
            test1_row.1, 100,
            "test1 value should be restored to original"
        );

        let test3_exists = restored_rows.iter().any(|(name, _)| name == "test3");
        assert!(test3_exists, "test3 should be restored");
    }
}
