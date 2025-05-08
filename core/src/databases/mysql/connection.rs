use std::{
    io::{Read, Write},
    process::Stdio,
    time::Duration,
};

use crate::databases::{
    connection::DatabaseConfig,
    version::{Version, VersionTrait},
    DatabaseConnectionTrait, DatabaseMetadata, UtilitiesTrait,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    MySql, Pool,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use super::{utilities::MySqlUtilities, version::MySqlVersion};

pub struct MySqlConnection {
    pub config: DatabaseConfig,
    pub pool: Pool<MySql>,
}

impl MySqlConnection {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let mut connect_options = MySqlConnectOptions::new()
            .host(&config.host)
            .username(&config.username)
            .database(&config.database)
            .port(config.port);

        connect_options = match &config.password {
            Some(password) => connect_options.password(&password),
            None => connect_options,
        };

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(connect_options)
            .await?;

        Ok(Self { config, pool })
    }

    async fn get_base_command(&self, bin_name: &str) -> Result<Command> {
        let metadata = self.get_metadata().await?;

        let version = match metadata.version {
            Version::MySql(version) => version,
            _ => return Err(anyhow!("Wrong version type")),
        };

        let utilities = MySqlUtilities::new(version);
        let mut cmd = utilities.get_command(bin_name).await?;

        if let Some(password) = &self.config.password {
            cmd.env("MYSQL_PWD", password.as_str());
        }

        Ok(cmd)
    }

    async fn get_command(&self, bin_name: &str) -> Result<Command> {
        let mut cmd = self.get_base_command(bin_name).await?;

        cmd.arg(format!("--host={}", self.config.host))
            .arg(format!("--port={}", self.config.port))
            .arg(format!("--user={}", self.config.username))
            .arg("--protocol=TCP")
            .arg(self.config.database.clone());

        Ok(cmd)
    }
}

#[async_trait]
impl DatabaseConnectionTrait for MySqlConnection {
    async fn get_metadata(&self) -> Result<DatabaseMetadata> {
        let version_string: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to get database version: {}", e))?;

        let version = match MySqlVersion::parse_string_version(version_string.0.as_str()) {
            Some(version) => version,
            None => return Err(anyhow!("Failed to parse MySQL version string")),
        };

        Ok(DatabaseMetadata {
            version: Version::MySql(version),
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
        let mut cmd = self.get_command("mysqldump").await?;

        cmd.arg("--opt")
            .arg("--single-transaction")
            .arg("--skip-lock-tables")
            .arg("--set-charset")
            .arg("--add-drop-database")
            .arg("--add-drop-table")
            .arg("--no-tablespaces")
            .arg("--skip-triggers");

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start mysqldump: {}", e))?;

        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture mysqldump stdout".to_string()))?;

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
                    return Err(anyhow!("Failed to read from mysqldump: {}", e));
                }
            }
        }

        let status = child
            .wait()
            .await
            .map_err(|e| anyhow!("mysqldump process failed: {}", e))?;

        if !status.success() {
            let mut stderr = child
                .stderr
                .take()
                .ok_or_else(|| anyhow!("Failed to capture mysqldump stderr".to_string()))?;

            let mut error_message = String::new();
            stderr
                .read_to_string(&mut error_message)
                .await
                .map_err(|e| anyhow!("Failed to read mysqldump stderr: {}", e))?;

            return Err(anyhow!("mysqldump failed: {}", error_message));
        }

        Ok(())
    }

    async fn restore(&self, reader: &mut (dyn Read + Send + Unpin)) -> Result<()> {
        let mut cmd = self.get_base_command("mysql").await?;

        cmd.arg(format!("--host={}", self.config.host))
            .arg(format!("--port={}", self.config.port))
            .arg(format!("--user={}", self.config.username))
            .arg("--protocol=TCP")
            .arg("-e")
            .arg(format!(
                "SELECT CONCAT('KILL ', id, ';') FROM information_schema.processlist 
                WHERE user = '{}' AND db = '{}' AND id != CONNECTION_ID();",
                self.config.username, self.config.database
            ));

        let drop_connections_output = cmd
            .output()
            .await
            .context(format!("Failed to execute connection termination command"))?;

        if !drop_connections_output.status.success() {
            let stderr = String::from_utf8_lossy(&drop_connections_output.stderr);
            let exit_code = drop_connections_output.status.code().unwrap_or(-1);

            return Err(anyhow!(
                "Failed to drop connections with exit code {}.\nError details: {}",
                exit_code,
                stderr.trim()
            ));
        }

        let mut cmd = self.get_command("mysql").await?;
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
            .map_err(|e| anyhow!("mysql process failed: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "mysql restore failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
