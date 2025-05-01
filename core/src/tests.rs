#[cfg(test)]
mod lib_tests {
    use anyhow::Result;
    use dotenv::dotenv;
    use serial_test::serial;
    use std::env;

    use crate::{
        backup,
        databases::configs::{MariaDBSourceConfig, PGSourceConfig, SourceConfig},
        is_connected, restore,
        storage::configs::{LocalStorageConfig, StorageConfig},
    };

    fn get_local_storage_config() -> Result<LocalStorageConfig> {
        let current_dir = env::current_dir()?;
        let test_backup_dir = current_dir.join("test-backups");

        Ok(LocalStorageConfig {
            name: "tests".into(),
            root: test_backup_dir.to_string_lossy().into_owned().into(),
            prefix: None,
        })
    }

    fn get_storage_config() -> Result<StorageConfig> {
        Ok(StorageConfig::Local(get_local_storage_config()?))
    }

    fn get_postgresql_source_config() -> Result<SourceConfig> {
        dotenv().ok();
        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;

        Ok(SourceConfig::PG(PGSourceConfig {
            name: "postgresql".into(),
            database: env::var("POSTGRESQL_NAME").unwrap_or_default(),
            host: env::var("POSTGRESQL_HOST").unwrap_or_default(),
            username: env::var("POSTGRESQL_USERNAME").unwrap_or_default(),
            port,
            password: Some(env::var("POSTGRESQL_PASSWORD").unwrap_or_default()),
            tunnel_config: None,
        }))
    }

    fn get_mariadb_source_config() -> Result<SourceConfig> {
        dotenv().ok();
        let port: u16 = env::var("MARIADB_PORT").unwrap_or("0".into()).parse()?;

        Ok(SourceConfig::MariaDB(MariaDBSourceConfig {
            name: "mariadb".into(),
            database: env::var("MARIADB_NAME").unwrap_or_default(),
            host: env::var("MARIADB_HOST").unwrap_or_default(),
            username: env::var("MARIADB_USERNAME").unwrap_or_default(),
            port,
            password: Some(env::var("MARIADB_PASSWORD").unwrap_or_default()),
            tunnel_config: None,
        }))
    }

    #[tokio::test]
    #[serial]
    async fn test_01_is_postgresql_connected() {
        let source_config =
            get_postgresql_source_config().expect("Failed to get PostgreSQL source config");

        let is_connected = is_connected(source_config)
            .await
            .expect("Failed to check PostgreSQL connection");

        assert_eq!(is_connected, true);
    }

    #[tokio::test]
    #[serial]
    async fn test_02_is_mariadb_connected() {
        let source_config =
            get_mariadb_source_config().expect("Failed to get MariaDB source config");

        let is_connected = is_connected(source_config)
            .await
            .expect("Failed to check MariaDB connection");

        assert_eq!(is_connected, true);
    }

    #[tokio::test]
    #[serial]
    async fn test_03_backup_postgresql() {
        let source_config =
            get_postgresql_source_config().expect("Failed to get PostgreSQL source config");

        let storage_config = get_storage_config().expect("Failed to get local storage config");

        let backup_key = backup(source_config, storage_config)
            .await
            .expect("Failed to check PostgreSQL connection");

        let root = get_local_storage_config()
            .expect("Failed to get local storage config")
            .root;

        let backup_path = root.join(backup_key);

        assert_eq!(backup_path.exists(), true);
    }

    #[tokio::test]
    #[serial]
    async fn test_04_backup_mariadb() {
        let source_config =
            get_mariadb_source_config().expect("Failed to get MariaDB source config");

        let storage_config = get_storage_config().expect("Failed to get local storage config");

        let backup_key = backup(source_config, storage_config)
            .await
            .expect("Failed to check MariaDB connection");

        let root = get_local_storage_config()
            .expect("Failed to get local storage config")
            .root;

        let backup_path = root.join(backup_key);

        assert_eq!(backup_path.exists(), true);
    }

    #[tokio::test]
    #[serial]
    async fn test_05_restore_postgresql() {
        let source_config =
            get_postgresql_source_config().expect("Failed to get PostgreSQL source config");

        let storage_config = get_storage_config().expect("Failed to get local storage config");

        let backup_key = backup(&source_config, &storage_config)
            .await
            .expect("Failed to check PostgreSQL connection");

        let root = get_local_storage_config()
            .expect("Failed to get local storage config")
            .root;

        let backup_path = root.join(&backup_key);

        assert_eq!(backup_path.exists(), true);

        restore(source_config, storage_config, backup_key.as_str(), true)
            .await
            .expect("Failed to restore database");
    }
}
