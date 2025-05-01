use super::commands::CommandBuilder;
use super::version::{PostgreSQLVersion, DEFAULT_POSTGRES_VERSION};
use crate::databases::DbVersion;
use anyhow::anyhow;
use anyhow::{Context, Result};
use bytes::Bytes;
use std::{env, fs};
use tempfile::TempDir;
use tokio::process::Command;

pub struct PostgreSQLTools {
    version: PostgreSQLVersion,
}

impl PostgreSQLTools {
    pub fn new(version: PostgreSQLVersion) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("pg_tools_cache");

        fs::create_dir_all(&cache_dir)?;

        Ok(PostgreSQLTools { version })
    }

    pub fn default() -> Result<Self> {
        Self::new(DEFAULT_POSTGRES_VERSION)
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

    pub async fn get_version(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<PostgreSQLVersion> {
        let cmd_builder =
            CommandBuilder::new(self.version, database, host, port, username, password)?;
        let mut cmd = cmd_builder.build_connection_command().await?;

        cmd.arg("SELECT version();");

        let output = cmd
            .output()
            .await
            .context("Failed to execute check version command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("PostgreSQL version check failed: {}", error));
        }

        let string_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let version = PostgreSQLVersion::parse_string_version(&string_version);

        match version {
            Some(version) => Ok(version),
            None => return Err(anyhow!("Failed to parse PostgreSQL version string")),
        }
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
                    "Failed to execute psql command: {}. Check if PostgreSQL client is installed.",
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
            } else if error.contains("could not connect") || error.contains("connection to server")
            {
                return Ok(false);
            }

            Err(anyhow!(
                "PostgreSQL connection check failed: {}",
                error.trim()
            ))
        }
    }

    pub async fn dump(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<Bytes> {
        let target_version = self
            .get_version(database, host, port, username, password)
            .await
            .context(format!(
                "Failed to connect to PostgreSQL at {}:{} to verify version",
                host, port
            ))?;

        if target_version != self.version {
            return Err(anyhow!(
                "Version mismatch: Tool is configured for PostgreSQL {} but target database is running {}. Please use the correct version.",
                self.version.as_str(),
                target_version.as_str(),
            ));
        }

        let cmd_builder =
            CommandBuilder::new(self.version, database, host, port, username, password)?;

        let mut cmd = cmd_builder.build_dump_command().await?;

        let output = cmd
            .output()
            .await
            .context(format!("Failed to execute pg_dump command"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            if stderr.contains("connection to server") && stderr.contains("failed") {
                return Err(anyhow!(
                    "Connection to database failed. Please check:\n- Host '{}' and port '{}' are correct\n- Database '{}' exists\n- User '{}' has access permissions\n- PostgreSQL is running\n\nError details: {}",
                    host, port, database, username, stderr.trim()
                ));
            } else if stderr.contains("permission denied") {
                return Err(anyhow!(
                    "Permission denied. User '{}' doesn't have sufficient privileges to dump database '{}'.\nError details: {}",
                    username, database, stderr.trim()
                ));
            } else {
                return Err(anyhow!(
                    "pg_dump failed with exit code {}.\nError details: {}",
                    exit_code,
                    stderr.trim()
                ));
            }
        }

        if output.stdout.is_empty() {
            return Err(anyhow!("pg_dump completed but didn't produce any output. This might indicate a problem with the dump process."));
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
            "postgres", // System database for PostgreSQL
            host,
            port,
            username,
            password,
        )?;

        let mut terminate_cmd = cmd_builder.build_connection_command().await?;
        terminate_cmd.arg(format!(
            "SELECT pg_terminate_backend(pg_stat_activity.pid) 
             FROM pg_stat_activity 
             WHERE pg_stat_activity.datname = '{}' 
             AND pid <> pg_backend_pid();",
            database
        ));

        let mut drop_cmd = cmd_builder.build_connection_command().await?;
        drop_cmd.arg(&format!("DROP DATABASE IF EXISTS \"{}\";", database));

        let mut create_cmd = cmd_builder.build_connection_command().await?;
        create_cmd.arg(&format!("CREATE DATABASE \"{}\";", database));

        terminate_cmd
            .output()
            .await
            .context(format!("Failed to execute connection termination command",))?;

        // We don't need to check the success of the terminate command - it's ok if there were no connections

        let drop_output = drop_cmd
            .output()
            .await
            .context(format!("Failed to execute psql drop command"))?;

        if !drop_output.status.success() {
            let stderr = String::from_utf8_lossy(&drop_output.stderr);
            let exit_code = drop_output.status.code().unwrap_or(-1);

            return Err(anyhow!(
                "Failed to drop database with exit code {}.\nError details: {}",
                exit_code,
                stderr.trim()
            ));
        }

        let create_output = create_cmd
            .output()
            .await
            .context(format!("Failed to execute psql create command"))?;

        if !create_output.status.success() {
            let stderr = String::from_utf8_lossy(&create_output.stderr);
            let exit_code = create_output.status.code().unwrap_or(-1);

            return Err(anyhow!(
                "Failed to create database with exit code {}.\nError details: {}",
                exit_code,
                stderr.trim()
            ));
        }

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
                "Failed to connect to PostgreSQL at {}:{} to verify version",
                host, port
            ))?;

        if target_version != self.version {
            return Err(anyhow!(
            "Version mismatch: Tool is configured for PostgreSQL {} but target database is running {}. Please use the correct version.",
            self.version.as_str(),
            target_version.as_str(),
        ));
        }

        if drop_database {
            self.drop_and_recreate_database(database, host, port, username, password)
                .await?;
        }

        let cmd_builder =
            CommandBuilder::new(self.version, database, host, port, username, password)?;
        let mut cmd = cmd_builder.build_restore_command().await?;

        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let dump_file_path = temp_dir.path().join("dump.pgdump");

        tokio::fs::write(&dump_file_path, &dump_data)
            .await
            .context("Failed to write dump data to temporary file")?;

        // Only add --clean and --if-exists if not dropping the entire database
        if !drop_database {
            cmd.arg("--clean") // Drop objects before recreating them
                .arg("--if-exists"); // Add IF EXISTS for cleaner drops
        }

        cmd.arg(&dump_file_path);

        let output = cmd
            .output()
            .await
            .context(format!("Failed to execute pg_restore command"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            if stderr.contains("connection to server") && stderr.contains("failed") {
                return Err(anyhow!(
                    "Connection to database failed. Please check:\n- Host '{}' and port '{}' are correct\n- Database '{}' exists\n- User '{}' has access permissions\n- PostgreSQL is running\n\nError details: {}",
                    host, port, database, username, stderr.trim()
                ));
            } else if stderr.contains("permission denied") {
                return Err(anyhow!(
                    "Permission denied. User '{}' doesn't have sufficient privileges to restore database '{}'.\nError details: {}",
                    username, database, stderr.trim()
                ));
            } else {
                return Err(anyhow!(
                    "pg_restore failed with exit code {}.\nError details: {}",
                    exit_code,
                    stderr.trim()
                ));
            }
        }

        Ok(())
    }
}
