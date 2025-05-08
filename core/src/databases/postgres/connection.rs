use std::{
    io::{Read, Write},
    process::Stdio,
    time::Duration,
};

use crate::databases::{
    connection::DatabaseConfig,
    version::{Version, VersionTrait},
    DatabaseMetadata, SQLDatabaseConnection, UtilitiesTrait,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Pool, Postgres,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use super::{utilities::Utilities, version::PostgreSQLVersionV2};

pub struct PostgreSQLConnection {
    config: DatabaseConfig,
    pool: Pool<Postgres>,
}

impl PostgreSQLConnection {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
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

    async fn get_base_command(&self, bin_name: &str) -> Result<Command> {
        let metadata = self.get_metadata().await?;
        let version = match metadata.version {
            Version::PostgreSQL(version) => version,
        };

        let utilities = Utilities::new(version);
        let mut cmd = utilities.get_command(bin_name).await?;

        if let Some(pass) = &self.config.password {
            cmd.env("PGPASSWORD", pass);
        }

        Ok(cmd)
    }

    async fn get_command(&self, bin_name: &str) -> Result<Command> {
        let mut cmd = self.get_base_command(bin_name).await?;

        cmd.arg("-h")
            .arg(&self.config.host)
            .arg("-p")
            .arg(self.config.port.to_string())
            .arg("-U")
            .arg(&self.config.username)
            .arg("-d")
            .arg(&self.config.database);

        Ok(cmd)
    }
}

#[async_trait]
impl SQLDatabaseConnection for PostgreSQLConnection {
    async fn get_metadata(&self) -> Result<DatabaseMetadata> {
        let version_string: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to get database version: {}", e))?;

        let version = match PostgreSQLVersionV2::parse_string_version(version_string.0.as_str()) {
            Some(version) => version,
            None => return Err(anyhow!("Fauiled to parse PostgreSQL version string")),
        };

        Ok(DatabaseMetadata {
            version: Version::PostgreSQL(version),
        })
    }

    async fn test(&self) -> Result<bool> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| true)
            .map_err(|e| anyhow!("Connection test failed: {}", e))
    }

    async fn backup(&self, writer: &mut (dyn Write + Send + Unpin)) -> Result<()> {
        let mut cmd = self.get_command("pg_dump").await?;

        cmd.arg("--format=custom")
            .arg("--format=plain")
            .arg("--encoding=UTF8")
            .arg("--schema=*")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--no-owner")
            .arg("--blobs")
            .arg("--exclude-schema=information_schema")
            .arg("--exclude-schema=pg_catalog")
            .arg("--exclude-schema=pg_toast")
            .arg("--exclude-schema=pg_temp*")
            .arg("--exclude-schema=pg_toast_temp*");

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start pg_dump: {}", e))?;

        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture pg_dump stdout".to_string()))?;

        let mut buffer = [0u8; 16384];

        loop {
            match stdout.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer
                        .write_all(&buffer[..n])
                        .map_err(|e| anyhow!("Failed to write backup data: {}", e))?;
                }
                Err(e) => {
                    return Err(anyhow!("Failed to read from pg_dump: {}", e));
                }
            }
        }

        let status = child
            .wait()
            .await
            .map_err(|e| anyhow!("pg_dump process failed: {}", e))?;

        if !status.success() {
            let mut stderr = child
                .stderr
                .take()
                .ok_or_else(|| anyhow!("Failed to capture pg_dump stderr".to_string()))?;

            let mut error_message = String::new();
            stderr
                .read_to_string(&mut error_message)
                .await
                .map_err(|e| anyhow!("Failed to read pg_dump stderr: {}", e))?;

            return Err(anyhow!("pg_dump failed: {}", error_message));
        }

        Ok(())
    }

    async fn restore(&self, reader: &mut (dyn Read + Send + Unpin)) -> Result<()> {
        let mut cmd = self.get_base_command("psql").await?;

        cmd.arg("-h")
            .arg(&self.config.host)
            .arg("-p")
            .arg(self.config.port.to_string())
            .arg("-U")
            .arg(&self.config.username)
            .arg("-d")
            .arg("postgres") // System database for PostgreSQL
            .arg("-c")
            .arg(format!(
                "SELECT pg_terminate_backend(pg_stat_activity.pid) 
                FROM pg_stat_activity 
                WHERE pg_stat_activity.datname = '{}' 
                AND pid <> pg_backend_pid();",
                self.config.database
            ));

        let drop_connections_output = cmd
            .output()
            .await
            .context(format!("Failed to execute connection termination command"))?;

        if !drop_connections_output.status.success() {
            let stderr = String::from_utf8_lossy(&drop_connections_output.stderr);
            let exit_code = drop_connections_output.status.code().unwrap_or(-1);

            return Err(anyhow!(
                "Failed to drop database with exit code {}.\nError details: {}",
                exit_code,
                stderr.trim()
            ));
        }

        let mut cmd = self.get_command("psql").await?;
        let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to capture psql stdin".to_string()))?;

        let mut buffer = [0u8; 16384];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    stdin.write_all(&buffer[..n]).await?;
                }
                Err(e) => {
                    return Err(anyhow!("Failed to read from pg_dump: {}", e));
                }
            }
        }

        drop(stdin);

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| anyhow!("psql process failed: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "psql restore failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod postgresql_connection_test {
    use super::*;
    use crate::databases::connection::ConnectionType;
    use dotenv::dotenv;
    use std::env;
    use std::thread::sleep;

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

        let version = match &metadata.version {
            Version::PostgreSQL(version) => Some(version),
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
        let connection = get_connection().await.expect("Failed to get connection");

        sqlx::query("DROP TABLE IF EXISTS backup_test_table")
            .execute(&connection.pool)
            .await
            .expect("Failed to drop test table");

        sqlx::query(
            "CREATE TABLE backup_test_table (id SERIAL PRIMARY KEY, name TEXT, value INTEGER)",
        )
        .execute(&connection.pool)
        .await
        .expect("Failed to create test table");

        sqlx::query("INSERT INTO backup_test_table (name, value) VALUES ('test1', 100), ('test2', 200), ('test3', 300)")
        .execute(&connection.pool)
        .await
        .expect("Failed to insert test data");

        let rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
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

        sqlx::query("UPDATE backup_test_table SET value = 999 WHERE name = 'test1'")
            .execute(&connection.pool)
            .await
            .expect("Failed to update test data");

        sqlx::query("DELETE FROM backup_test_table WHERE name = 'test3'")
            .execute(&connection.pool)
            .await
            .expect("Failed to delete test data");

        let modified_rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
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

        let restored_rows: Vec<(String, i32)> =
            sqlx::query_as("SELECT name, value FROM backup_test_table ORDER BY id")
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
