use crate::databases::DbVersion;

use super::commands::CommandBuilder;
use super::version::{MySQLVersion, DEFAULT_MYSQL_VERSION};
use anyhow::anyhow;
use anyhow::{Context, Result};
use bytes::Bytes;
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;

pub struct MySQLTools {
    pub version: MySQLVersion,
}

impl MySQLTools {
    pub fn new(version: MySQLVersion) -> Self {
        return MySQLTools { version };
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_MYSQL_VERSION)
    }

    pub async fn with_detected_version(
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<Self> {
        let tools = Self::default();
        let version = tools
            .get_version(database, host, port, username, password)
            .await?;
        let tools = Self::new(version);
        return Ok(tools);
    }

    pub async fn is_connected(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<bool> {
        let cmd_builder =
            CommandBuilder::new(self.version, database, host, port, username, password)?;
        let mut cmd = cmd_builder.build_connection_check_command().await?;

        let output = match cmd.output().await {
            Ok(output) => output,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to execute mysql command: {}. Check if MySQL/MariaDB client is installed.",
                    e
                ));
            }
        };

        if output.status.success() {
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);

            if error.contains("timeout") {
                return Ok(false);
            } else if error.contains("Can't connect") || error.contains("Access denied") {
                return Ok(false);
            }

            Err(anyhow!("MariaDB connection check failed: {}", error.trim()))
        }
    }

    pub async fn get_version(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<MySQLVersion> {
        let cmd_builder =
            CommandBuilder::new(self.version, database, host, port, username, password)?;
        let mut cmd = cmd_builder.build_version_check_command().await?;

        let output = cmd
            .output()
            .await
            .context("Failed to execute mysql command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("MariaDB version check failed: {}", error));
        }

        let string_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let version = MySQLVersion::parse_string_version(&string_version);

        match version {
            Some(version) => Ok(version),
            None => return Err(anyhow!("Failed to parse MariaDB version string")),
        }
    }

    pub async fn get_connection(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<Command> {
        let cmd_builder =
            CommandBuilder::new(self.version, database, host, port, username, password)?;
        let cmd = cmd_builder.build_connection_command().await?;

        Ok(cmd)
    }

    pub async fn dump(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<Bytes> {
        let cmd_builder = CommandBuilder::new(
            self.version.clone(),
            database,
            host,
            port,
            username,
            password,
        )?;

        let mut cmd = cmd_builder.build_dump_command().await?;

        let output = cmd
            .output()
            .await
            .context("Failed to execute mariadb-dump command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            if stderr.contains("Can't connect") {
                return Err(anyhow!(
                    "Connection to database failed. Please check:\n- Host '{}' and port '{}' are correct\n- Database '{}' exists\n- User '{}' has access permissions\n- MariaDB is running\n\nError details: {}",
                    host, port, database, username, stderr.trim()
                ));
            } else if stderr.contains("Access denied") {
                return Err(anyhow!(
                    "Permission denied. User '{}' doesn't have sufficient privileges to dump database '{}'.\nError details: {}",
                    username, database, stderr.trim()
                ));
            } else {
                return Err(anyhow!(
                    "mariadb-dump failed with exit code {}.\nError details: {}",
                    exit_code,
                    stderr.trim()
                ));
            }
        }

        if output.stdout.is_empty() {
            return Err(anyhow!(
                "mariadb-dump completed but didn't produce any output. This might indicate a problem with the dump process or an empty database."
            ));
        }

        Ok(Bytes::from(output.stdout))
    }

    pub async fn drop_and_recreate_database(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<()> {
        let cmd_builder = CommandBuilder::new(
            self.version.clone(),
            "mysql", // System database for MariaDB
            host,
            port,
            username,
            password,
        )?;

        // First, kill all connections to the target database
        let kill_sql = format!(
            "SELECT concat('KILL ', id, ';') FROM information_schema.processlist 
             WHERE db = '{}' AND id <> connection_id();",
            database
        );

        // Get the kill commands as a series of statements
        let mut kill_cmd = cmd_builder.build_connection_command().await?;
        kill_cmd.arg(&kill_sql);

        let kill_output = kill_cmd
            .output()
            .await
            .context(format!("Failed to execute connection termination command"))?;

        if !kill_output.status.success() {
            // Log the error but continue - it's ok if there were no connections to kill
            println!(
                "Note: Connection termination returned non-zero: {}",
                String::from_utf8_lossy(&kill_output.stderr)
            );
        } else {
            // If there were connections to kill, execute the kill commands
            let kill_commands = String::from_utf8_lossy(&kill_output.stdout);

            if !kill_commands.trim().is_empty() {
                let mut exec_kills = cmd_builder.build_connection_command().await?;
                exec_kills.arg(kill_commands.to_string());

                let _ = exec_kills
                    .output()
                    .await
                    .context("Failed to execute kill commands")?;

                // Brief pause to let connections terminate
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        }

        // Set up command for dropping the database
        let mut drop_cmd = cmd_builder.build_connection_command().await?;
        drop_cmd.arg(format!("DROP DATABASE IF EXISTS `{}`;", database));

        // Execute the drop command
        let drop_output = drop_cmd
            .output()
            .await
            .context("Failed to execute MariaDB drop command")?;

        // Check if the drop command executed successfully
        if !drop_output.status.success() {
            let stderr = String::from_utf8_lossy(&drop_output.stderr);
            let exit_code = drop_output.status.code().unwrap_or(-1);

            return Err(anyhow!(
                "Failed to drop database with exit code {}.\nError details: {}",
                exit_code,
                stderr.trim()
            ));
        }

        // Set up command for creating the database
        let mut create_cmd = cmd_builder.build_connection_command().await?;
        create_cmd.arg(format!(
            "CREATE DATABASE `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
            database
        ));

        // Execute the create command
        let create_output = create_cmd
            .output()
            .await
            .context("Failed to execute MariaDB create command")?;

        // Check if the create command executed successfully
        if !create_output.status.success() {
            let stderr = String::from_utf8_lossy(&create_output.stderr);
            let exit_code = create_output.status.code().unwrap_or(-1);

            return Err(anyhow!(
                "Failed to create database with exit code {}.\nError details: {}",
                exit_code,
                stderr.trim()
            ));
        }

        // Wait a moment to ensure the database is ready
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok(())
    }

    pub async fn restore(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
        dump_data: Bytes,
        drop_database: bool,
    ) -> Result<()> {
        let target_version = self
            .get_version(database, host, port, username, password)
            .await
            .context(format!(
                "Failed to connect to MariaDB at {}:{} to verify version",
                host, port
            ))?;

        if target_version != self.version {
            return Err(anyhow!(
            "Version mismatch: Tool is configured for MariaDB {} but target database is running {}. Please use the correct version.",
            self.version.as_str(),
            target_version.as_str(),
        ));
        }

        if drop_database {
            self.drop_and_recreate_database(database, host, port, username, password)
                .await?;
        }

        let cmd_builder = CommandBuilder::new(
            self.version.clone(),
            database,
            host,
            port,
            username,
            password,
        )?;

        // Create a temporary directory to store the SQL file
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let sql_file_path = temp_dir.path().join("dump.sql");

        // Write the raw SQL data to a temporary file
        fs::write(&sql_file_path, &dump_data)
            .await
            .context(format!(
                "Failed to write SQL data to file: {}",
                sql_file_path.display()
            ))?;

        // Build the mariadb command to import the SQL file
        let mariadb_path = cmd_builder.get_bin_path().await?.join("mariadb");

        // Create a shell command that pipes the SQL file into mariadb
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(format!(
            "{} --host={} --port={} --user={} {} < {}",
            mariadb_path.display(),
            host,
            port,
            username,
            database,
            sql_file_path.display()
        ));

        // Set the MYSQL_PWD environment variable if password is provided
        if let Some(pass) = password {
            cmd.env("MYSQL_PWD", pass);
        }

        // Execute the command
        let output = cmd.output().await.context(format!(
            "Failed to execute mariadb restore command using {}",
            mariadb_path.display()
        ))?;

        // Check if the command executed successfully
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            // Provide specific error messages based on common failure patterns
            if stderr.contains("Can't connect to MySQL server") {
                return Err(anyhow!(
                    "Connection to database failed. Please check:\n- Host '{}' and port '{}' are correct\n- Database '{}' exists\n- User '{}' has access permissions\n- MariaDB is running\n\nError details: {}",
                    host, port, database, username, stderr.trim()
                ));
            } else if stderr.contains("Access denied") {
                return Err(anyhow!(
                    "Permission denied. User '{}' doesn't have sufficient privileges to restore database '{}'.\nError details: {}",
                    username, database, stderr.trim()
                ));
            } else {
                return Err(anyhow!(
                    "MariaDB restore failed with exit code {}.\nCommand: {} --host={} --port={} --user={} {} < {}\n\nError details: {}",
                    exit_code,
                    mariadb_path.display(),
                    host,
                    port,
                    username,
                    database,
                    sql_file_path.display(),
                    stderr.trim()
                ));
            }
        }

        Ok(())
    }
}
