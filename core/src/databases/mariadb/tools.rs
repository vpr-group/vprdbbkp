use super::commands::CommandBuilder;
use super::version::{MariaDBVersion, DEFAULT_MARIADB_VERSION};
use anyhow::anyhow;
use anyhow::{Context, Result};
use bytes::Bytes;
use std::process::Stdio;
use tokio::process::Command;

pub struct MariaDBTools {
    pub version: MariaDBVersion,
}

impl MariaDBTools {
    pub fn new(version: MariaDBVersion) -> Self {
        return MariaDBTools { version };
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_MARIADB_VERSION)
    }

    pub async fn with_detected_version(
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
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
        password: &str,
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
        password: &str,
    ) -> Result<MariaDBVersion> {
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
        let version = MariaDBVersion::parse_string_version(&string_version);

        match version {
            Some(version) => Ok(version),
            None => return Err(anyhow!("Failed to parse MariaDB version string")),
        }
    }

    pub async fn dump(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        compression: Option<u8>,
    ) -> Result<Bytes> {
        let cmd_builder =
            CommandBuilder::new(self.version, database, host, port, username, password)?;
        let mut cmd = cmd_builder.build_dump_command(compression).await?;

        if let Some(compression) = compression {
            // Start mysqldump process
            let mut mysqldump_process = cmd.spawn().context("Failed to execute mysqldump")?;

            // Set up gzip command
            let mut gzip_cmd = Command::new("gzip");
            gzip_cmd
                .arg(format!("-{}", compression))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());

            // Start gzip process
            let mut gzip_process = gzip_cmd.spawn().context("Failed to execute gzip")?;

            // Get handles to the stdout/stdin of both processes
            let mysqldump_stdout = mysqldump_process
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture mysqldump stdout"))?;
            let gzip_stdin = gzip_process
                .stdin
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdin"))?;

            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                // Convert the stdout of mysqldump to a tokio readable
                let mysqldump_stdout = mysqldump_stdout;

                // Convert the stdin of gzip to a tokio writable
                let mut gzip_stdin = gzip_stdin;

                // Copy data from mysqldump to gzip
                let mut buffer = vec![0; 8192]; // 8KB buffer
                let mut reader = tokio::io::BufReader::new(mysqldump_stdout);

                loop {
                    let n = reader
                        .read(&mut buffer)
                        .await
                        .expect("Failed to read from mysqldump");
                    if n == 0 {
                        break; // End of input
                    }
                    gzip_stdin
                        .write_all(&buffer[..n])
                        .await
                        .expect("Failed to write to gzip");
                }

                // Close the stdin to signal EOF to gzip
                drop(gzip_stdin);
            });

            // Wait for both processes to complete
            let (mysqldump_status, output) =
                tokio::join!(mysqldump_process.wait(), gzip_process.wait_with_output());

            let mysqldump_status = mysqldump_status.context("Failed to wait for mysqldump")?;
            let output = output.context("Failed to read gzip output")?;

            if !mysqldump_status.success() {
                let exit_code = mysqldump_status.code().unwrap_or(-1);
                return Err(anyhow!("mysqldump failed with exit code: {}", exit_code));
            }

            // Check gzip process exit status
            if !output.status.success() {
                let exit_code = output.status.code().unwrap_or(-1);
                return Err(anyhow!("gzip failed with exit code: {}", exit_code));
            }

            Ok(Bytes::from(output.stdout))
        } else {
            // Execute the command
            let output = cmd
                .output()
                .await
                .context("Failed to execute mysqldump command at ")?;

            // Check if the command executed successfully
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                // Provide specific error messages based on common failure patterns
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
                        "mysqldump failed with exit code {}.\nCommand: --host={} --port={} --user={} --single-transaction --routines --triggers --events --complete-insert --compress {}\n\nError details: {}",
                        exit_code,
                        host,
                        port,
                        username,
                        database,
                        stderr.trim()
                    ));
                }
            }

            // Check if we received any data
            if output.stdout.is_empty() {
                return Err(anyhow!("mysqldump completed but didn't produce any output. This might indicate a problem with the dump process or an empty database."));
            }

            Ok(Bytes::from(output.stdout))
        }
    }
}
