#[cfg(test)]
mod postgresql_connection_test {
    use crate::databases::postgres::connection::PostgreSqlConnection;
    use crate::databases::ssh_tunnel::{SshAuthMethod, SshTunnelConfig};
    use crate::databases::version::Version;
    use crate::databases::{
        ConnectionType, DatabaseConfig, DatabaseConnectionTrait, RestoreOptions,
    };
    use crate::test_utils::test_utils::{
        get_postgresql_connection, get_postgresql_pool, initialize_test,
    };

    use anyhow::Result;
    use dotenv::dotenv;
    use std::env;
    use std::thread::sleep;
    use std::time::Duration;

    async fn get_tunneled_connection() -> Result<PostgreSqlConnection> {
        dotenv().ok();

        let port: u16 = env::var("DB_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("DB_PASSWORD").unwrap_or_default();

        let config = DatabaseConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: ConnectionType::PostgreSql,
            host: "localhost".into(),
            password: Some(password),
            username: env::var("DB_USERNAME").unwrap_or_default(),
            database: env::var("DB_NAME").unwrap_or_default(),
            port,
            ssh_tunnel: Some(SshTunnelConfig {
                host: env::var("SSH_HOST").unwrap_or_default(),
                username: env::var("SSH_USERNAME").unwrap_or_default(),
                port: 22,
                auth_method: SshAuthMethod::PrivateKey {
                    key_path: env::var("SSH_KEY_PATH").unwrap_or_default(),
                    passphrase_key: None,
                },
            }),
        };

        let connection = PostgreSqlConnection::new(config).await?;

        Ok(connection)
    }

    #[tokio::test]
    async fn test_01_connection_test() {
        initialize_test();
        let connection = get_postgresql_connection(false)
            .await
            .expect("Failed to get connection");
        let is_connected = connection.test().await.expect("Failed to test connection");
        assert!(is_connected)
    }

    #[tokio::test]
    async fn test_02_get_metadata() {
        initialize_test();
        let connection = get_postgresql_connection(false)
            .await
            .expect("Failed to get connection");
        let metadata = connection
            .get_metadata()
            .await
            .expect("Failed to get metadata");

        let version = match &metadata.version {
            Version::PostgreSQL(version) => Some(version),
            _ => None,
        };

        assert!(version.is_some());

        let version = version.unwrap();

        assert!(version.to_string().contains("15"));
    }

    #[tokio::test]
    async fn test_03_dump() {
        initialize_test();
        let connection = get_postgresql_connection(false)
            .await
            .expect("Failed to get connection");

        let mut buffer = Vec::new();
        connection
            .backup(&mut buffer)
            .await
            .expect("Failed to backup database");

        assert!(!buffer.is_empty());
    }

    #[tokio::test]
    async fn test_04_restore() {
        initialize_test();
        let test_table_name = format!("test_restore_{}", chrono::Utc::now().timestamp());
        let db_pool = get_postgresql_pool().await.expect("Failed to get db_pool");
        let connection = get_postgresql_connection(false)
            .await
            .expect("Failed to get connection");

        sqlx::query(format!("DROP TABLE IF EXISTS {}", test_table_name).as_str())
            .execute(&db_pool)
            .await
            .expect("Failed to drop test table");

        sqlx::query(
            format!(
                "CREATE TABLE {} (id SERIAL PRIMARY KEY, name TEXT, value INTEGER)",
                test_table_name
            )
            .as_str(),
        )
        .execute(&db_pool)
        .await
        .expect("Failed to create test table");

        sqlx::query(
            format!("INSERT INTO {} (name, value) VALUES ('test1', 100), ('test2', 200), ('test3', 300)", test_table_name).as_str(),
        )
        .execute(&db_pool)
        .await
        .expect("Failed to insert test data");

        let rows: Vec<(String, i32)> = sqlx::query_as(
            format!("SELECT name, value FROM {} ORDER BY id", test_table_name).as_str(),
        )
        .fetch_all(&db_pool)
        .await
        .expect("Failed to fetch test data");

        assert_eq!(rows.len(), 3, "Should have 3 rows before backup");

        let mut backup_buffer = Vec::new();
        connection
            .backup(&mut backup_buffer)
            .await
            .expect("Failed to backup database");

        assert!(!backup_buffer.is_empty(), "Backup should not be empty");

        println!("{}", String::from_utf8(backup_buffer.clone()).unwrap());

        println!("ldxkjhflkjdfhklsjhglkjshdlfkjgh");

        sqlx::query(
            format!(
                "UPDATE {} SET value = 999 WHERE name = 'test1'",
                test_table_name
            )
            .as_str(),
        )
        .execute(&db_pool)
        .await
        .expect("Failed to update test data");

        sqlx::query(format!("DELETE FROM {} WHERE name = 'test3'", test_table_name).as_str())
            .execute(&db_pool)
            .await
            .expect("Failed to delete test data");

        let modified_rows: Vec<(String, i32)> = sqlx::query_as(
            format!("SELECT name, value FROM {} ORDER BY id", test_table_name).as_str(),
        )
        .fetch_all(&db_pool)
        .await
        .expect("Failed to fetch modified data");

        assert_eq!(modified_rows.len(), 2, "Should have 2 rows after deletion");
        assert_eq!(modified_rows[0].1, 999, "Value should be modified");

        sleep(Duration::from_secs(1));

        let restore_connection = get_postgresql_connection(false)
            .await
            .expect("Failed to get connection for restore");

        let mut backup_cursor = std::io::Cursor::new(backup_buffer);

        restore_connection
            .restore_with_options(
                &mut backup_cursor,
                RestoreOptions {
                    drop_database_first: false,
                },
            )
            .await
            .expect("Failed to restore database");

        let verify_pool = get_postgresql_pool()
            .await
            .expect("Failed to get connection after restore");

        let restored_rows: Vec<(String, i32)> = sqlx::query_as(
            format!("SELECT name, value FROM {} ORDER BY id", test_table_name).as_str(),
        )
        .fetch_all(&verify_pool)
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
    async fn test_05_tunneled_connection() {
        initialize_test();
        let connection = get_tunneled_connection()
            .await
            .expect("Failed to get tunneled connection");

        let is_connected = connection.test().await.expect("Failed to check connection");

        assert!(is_connected);
    }

    #[ignore]
    #[tokio::test]
    async fn test_05_tunneled_backup() {
        initialize_test();
        let connection = get_tunneled_connection()
            .await
            .expect("Failed to get tunneled connection");

        let mut buf = vec![];

        connection
            .backup(&mut buf)
            .await
            .expect("Failed to check connection");

        assert!(buf.len() > 0);
    }
}
