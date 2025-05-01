#[cfg(test)]
mod lib_tests {
    use anyhow::Result;
    use dotenv::dotenv;
    use serial_test::serial;
    use std::env;
    use tokio::process::Command;

    use crate::{
        backup,
        databases::{
            configs::{MariaDBSourceConfig, PGSourceConfig, SourceConfig},
            mariadb::{tools::MariaDBTools, MariaDB},
            postgres::{tools::PostgreSQLTools, PostgreSQL},
        },
        is_connected, restore,
        storage::configs::{LocalStorageConfig, StorageConfig},
    };

    struct ConnectionOptions {
        host: String,
        port: u16,
        user: String,
        password: String,
        db_name: String,
    }

    fn get_postgresql_connection_options() -> Result<ConnectionOptions> {
        dotenv().ok();

        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;

        Ok(ConnectionOptions {
            host: env::var("POSTGRESQL_HOST").unwrap_or_default(),
            password: env::var("POSTGRESQL_PASSWORD").unwrap_or_default(),
            user: env::var("POSTGRESQL_USERNAME").unwrap_or_default(),
            db_name: env::var("POSTGRESQL_NAME").unwrap_or_default(),
            port,
        })
    }

    fn get_mariadb_connection_options() -> Result<ConnectionOptions> {
        dotenv().ok();

        let port: u16 = env::var("MARIADB_PORT").unwrap_or("0".into()).parse()?;

        Ok(ConnectionOptions {
            host: env::var("MARIADB_HOST").unwrap_or_default(),
            password: env::var("MARIADB_PASSWORD").unwrap_or_default(),
            user: env::var("MARIADB_USERNAME").unwrap_or_default(),
            db_name: env::var("MARIADB_NAME").unwrap_or_default(),
            port,
        })
    }

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

        let options = get_postgresql_connection_options()?;

        Ok(SourceConfig::PG(PGSourceConfig {
            name: "postgresql".into(),
            database: options.db_name,
            host: options.host,
            username: options.user,
            port: options.port,
            password: Some(options.password),
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

    fn get_postgresql() -> Result<PostgreSQL> {
        let options = get_postgresql_connection_options()?;

        Ok(PostgreSQL::new(
            options.db_name.as_str(),
            options.host.as_str(),
            options.port,
            options.user.as_str(),
            Some(options.password.as_str()),
        ))
    }

    async fn get_postgresql_tools() -> Result<PostgreSQLTools> {
        let postgresql = get_postgresql()?;
        let tools = postgresql.get_tools().await?;

        Ok(tools)
    }

    async fn get_postgresql_connection() -> Result<Command> {
        let options = get_postgresql_connection_options()?;
        let tools = get_postgresql_tools().await?;
        let connection = tools
            .get_connection(
                &options.db_name,
                &options.host,
                options.port,
                &options.user,
                Some(options.password.as_str()),
            )
            .await?;

        Ok(connection)
    }

    fn get_mariadb() -> Result<MariaDB> {
        let options = get_mariadb_connection_options()?;

        Ok(MariaDB::new(
            options.db_name.as_str(),
            options.host.as_str(),
            options.port,
            options.user.as_str(),
            Some(options.password.as_str()),
        ))
    }

    async fn get_mariadb_tools() -> Result<MariaDBTools> {
        let mariadb = get_mariadb()?;
        let tools = mariadb.get_tools().await?;

        Ok(tools)
    }

    async fn get_mariadb_connection() -> Result<Command> {
        let options = get_mariadb_connection_options()?;
        let tools = get_mariadb_tools().await?;
        let connection = tools
            .get_connection(
                &options.db_name,
                &options.host,
                options.port,
                &options.user,
                Some(options.password.as_str()),
            )
            .await?;

        Ok(connection)
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

        let test_table_name = format!("test_lib_restore_{}", chrono::Utc::now().timestamp());
        let mut connection = get_postgresql_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let create_table_cmd = connection
            .arg(format!(
                "CREATE TABLE {} (id INT PRIMARY KEY, value VARCHAR(255))",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute create table command");

        if !create_table_cmd.status.success() {
            panic!(
                "Failed to create test table: {}",
                String::from_utf8_lossy(&create_table_cmd.stderr)
            );
        }

        connection = get_postgresql_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let insert_data_cmd = connection
            .arg(format!(
                "INSERT INTO {} VALUES (1, 'test_value')",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute insert command");

        if !insert_data_cmd.status.success() {
            panic!(
                "Failed to insert test data: {}",
                String::from_utf8_lossy(&insert_data_cmd.stderr)
            );
        }

        connection = get_postgresql_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        if !select_data_cmd.status.success() {
            panic!(
                "Failed to query test data: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        let output = String::from_utf8_lossy(&select_data_cmd.stdout)
            .trim()
            .to_string();

        assert_eq!(
            output, "test_value",
            "Value in database doesn't match expected value"
        );

        let backup_key = backup(&source_config, &storage_config)
            .await
            .expect("Failed to check PostgreSQL connection");

        let root = get_local_storage_config()
            .expect("Failed to get local storage config")
            .root;

        let backup_path = root.join(&backup_key);

        assert_eq!(backup_path.exists(), true);

        connection = get_postgresql_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let drop_table_cmd = connection
            .arg(format!("DROP table {}", test_table_name))
            .output()
            .await
            .expect("Failed to execture drop table command");

        if !drop_table_cmd.status.success() {
            panic!(
                "Failed to drop table: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        connection = get_postgresql_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        // Expect the query to fail as the table has been dropped
        assert_eq!(select_data_cmd.status.success(), false);

        restore(source_config, storage_config, backup_key.as_str(), true)
            .await
            .expect("Failed to restore database");

        connection = get_postgresql_connection()
            .await
            .expect("Failed to get PostgreSQL connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        if !select_data_cmd.status.success() {
            panic!(
                "Failed to query test data: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        let output = String::from_utf8_lossy(&select_data_cmd.stdout)
            .trim()
            .to_string();

        assert_eq!(
            output, "test_value",
            "Value in database doesn't match expected value"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_06_restore_mariadb() {
        let source_config =
            get_mariadb_source_config().expect("Failed to get MariaDB source config");

        let storage_config = get_storage_config().expect("Failed to get local storage config");

        let test_table_name = format!("test_lib_restore_{}", chrono::Utc::now().timestamp());
        let mut connection = get_mariadb_connection()
            .await
            .expect("Failed to get MariaDB connection");

        let create_table_cmd = connection
            .arg(format!(
                "CREATE TABLE {} (id INT PRIMARY KEY, value VARCHAR(255))",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute create table command");

        if !create_table_cmd.status.success() {
            panic!(
                "Failed to create test table: {}",
                String::from_utf8_lossy(&create_table_cmd.stderr)
            );
        }

        connection = get_mariadb_connection()
            .await
            .expect("Failed to get MariaDB connection");

        let insert_data_cmd = connection
            .arg(format!(
                "INSERT INTO {} VALUES (1, 'test_value')",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute insert command");

        if !insert_data_cmd.status.success() {
            panic!(
                "Failed to insert test data: {}",
                String::from_utf8_lossy(&insert_data_cmd.stderr)
            );
        }

        connection = get_mariadb_connection()
            .await
            .expect("Failed to get MariaDB connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        if !select_data_cmd.status.success() {
            panic!(
                "Failed to query test data: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        let output = String::from_utf8_lossy(&select_data_cmd.stdout)
            .trim()
            .to_string();

        assert_eq!(
            output, "test_value",
            "Value in database doesn't match expected value"
        );

        let backup_key = backup(&source_config, &storage_config)
            .await
            .expect("Failed to check PostgreSQL connection");

        let root = get_local_storage_config()
            .expect("Failed to get local storage config")
            .root;

        let backup_path = root.join(&backup_key);

        assert_eq!(backup_path.exists(), true);

        connection = get_mariadb_connection()
            .await
            .expect("Failed to get MariaDB connection");

        let drop_table_cmd = connection
            .arg(format!("DROP table {}", test_table_name))
            .output()
            .await
            .expect("Failed to execture drop table command");

        if !drop_table_cmd.status.success() {
            panic!(
                "Failed to drop table: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        connection = get_mariadb_connection()
            .await
            .expect("Failed to get MariaDB connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        // Expect the query to fail as the table has been dropped
        assert_eq!(select_data_cmd.status.success(), false);

        restore(source_config, storage_config, backup_key.as_str(), true)
            .await
            .expect("Failed to restore database");

        connection = get_mariadb_connection()
            .await
            .expect("Failed to get MariaDB connection");

        let select_data_cmd = connection
            .arg(format!(
                "SELECT value FROM {} WHERE id = 1",
                test_table_name
            ))
            .output()
            .await
            .expect("Failed to execute select command");

        if !select_data_cmd.status.success() {
            panic!(
                "Failed to query test data: {}",
                String::from_utf8_lossy(&select_data_cmd.stderr)
            );
        }

        let output = String::from_utf8_lossy(&select_data_cmd.stdout)
            .trim()
            .to_string();

        assert_eq!(
            output, "test_value",
            "Value in database doesn't match expected value"
        );
    }
}
