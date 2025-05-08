#[cfg(test)]
mod postgresql_connection_test {
    use crate::databases::postgres::connection::PostgreSqlConnection;
    use crate::databases::version::Version;
    use crate::databases::{ConnectionType, DatabaseConfig, DatabaseConnectionTrait};
    use anyhow::Result;
    use dotenv::dotenv;
    use std::env;
    use std::thread::sleep;
    use std::time::Duration;

    async fn get_connection() -> Result<PostgreSqlConnection> {
        dotenv().ok();

        let port: u16 = env::var("POSTGRESQL_PORT").unwrap_or("0".into()).parse()?;
        let password = env::var("POSTGRESQL_PASSWORD").unwrap_or_default();
        let connection = PostgreSqlConnection::new(DatabaseConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: ConnectionType::PostgreSql,
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

        let version = match &metadata.version {
            Version::PostgreSQL(version) => Some(version),
            _ => None,
        };

        assert!(version.is_some());

        let version = version.unwrap();

        assert!(version.to_string().contains("15.12"));
    }

    #[tokio::test]
    async fn test_03_dump() {
        let connection = get_connection().await.expect("Failed to get connection");

        let mut buffer = Vec::new();
        connection
            .backup(&mut buffer)
            .await
            .expect("Failed to backup database");

        assert!(!buffer.is_empty());
    }

    #[tokio::test]
    async fn test_04_restore() {
        let test_table_name = format!("test_restore_{}", chrono::Utc::now().timestamp());
        let connection = get_connection().await.expect("Failed to get connection");

        sqlx::query(format!("DROP TABLE IF EXISTS {}", test_table_name).as_str())
            .execute(&connection.pool)
            .await
            .expect("Failed to drop test table");

        sqlx::query(
            format!(
                "CREATE TABLE {} (id SERIAL PRIMARY KEY, name TEXT, value INTEGER)",
                test_table_name
            )
            .as_str(),
        )
        .execute(&connection.pool)
        .await
        .expect("Failed to create test table");

        sqlx::query(
            format!("INSERT INTO {} (name, value) VALUES ('test1', 100), ('test2', 200), ('test3', 300)", test_table_name).as_str(),
        )
        .execute(&connection.pool)
        .await
        .expect("Failed to insert test data");

        let rows: Vec<(String, i32)> = sqlx::query_as(
            format!("SELECT name, value FROM {} ORDER BY id", test_table_name).as_str(),
        )
        .fetch_all(&connection.pool)
        .await
        .expect("Failed to fetch test data");

        assert_eq!(rows.len(), 3, "Should have 3 rows before backup");

        let mut backup_buffer = Vec::new();
        connection
            .backup(&mut backup_buffer)
            .await
            .expect("Failed to backup database");

        assert!(!backup_buffer.is_empty(), "Backup should not be empty");

        sqlx::query(
            format!(
                "UPDATE {} SET value = 999 WHERE name = 'test1'",
                test_table_name
            )
            .as_str(),
        )
        .execute(&connection.pool)
        .await
        .expect("Failed to update test data");

        sqlx::query(format!("DELETE FROM {} WHERE name = 'test3'", test_table_name).as_str())
            .execute(&connection.pool)
            .await
            .expect("Failed to delete test data");

        let modified_rows: Vec<(String, i32)> = sqlx::query_as(
            format!("SELECT name, value FROM {} ORDER BY id", test_table_name).as_str(),
        )
        .fetch_all(&connection.pool)
        .await
        .expect("Failed to fetch modified data");

        assert_eq!(modified_rows.len(), 2, "Should have 2 rows after deletion");
        assert_eq!(modified_rows[0].1, 999, "Value should be modified");

        sleep(Duration::from_secs(1));

        let restore_connection = get_connection()
            .await
            .expect("Failed to get connection for restore");

        let mut backup_cursor = std::io::Cursor::new(backup_buffer);

        restore_connection
            .restore(&mut backup_cursor)
            .await
            .expect("Failed to restore database");

        let verify_connection = get_connection()
            .await
            .expect("Failed to get connection after restore");

        let restored_rows: Vec<(String, i32)> = sqlx::query_as(
            format!("SELECT name, value FROM {} ORDER BY id", test_table_name).as_str(),
        )
        .fetch_all(&verify_connection.pool)
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
