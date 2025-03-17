use super::pg_utils::get_postgres_base_directory;
use super::pg_versions::{PostgresVersion, DEFAULT_POSTGRES_VERSION};
use crate::databases::postgres::pg_installer::PgInstaller;
use anyhow::anyhow;
use anyhow::{Context, Result};
use bytes::Bytes;
use std::path::Path;
use std::process::Stdio;
use std::{env, fs, path::PathBuf};
use tempfile::TempDir;
use tokio::process::Command;

pub struct PgTools {
    cache_dir: PathBuf,
    version: PostgresVersion,
}

impl PgTools {
    pub fn new(version: PostgresVersion) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("pg_tools_cache");

        fs::create_dir_all(&cache_dir)?;

        Ok(PgTools { cache_dir, version })
    }

    pub fn default() -> Result<Self> {
        Self::new(DEFAULT_POSTGRES_VERSION)
    }

    pub async fn get_pg_base_path(&self) -> Result<PathBuf> {
        let default_path = get_postgres_base_directory();

        // Check if version is installed in default folder
        if let Ok(path) = default_path {
            let default_version_path = &path.join(format!("{}/bin", self.version.as_str()));

            if Path::new(default_version_path).exists() {
                return Ok(path);
            }
        }

        // Check if there is a version available from cache dir
        if Path::new(
            &self
                .cache_dir
                .join(format!("{}/bin", self.version.as_str())),
        )
        .exists()
        {
            return Ok(self.cache_dir.clone());
        } else {
            // If there is no available version try to install one
            let installer = PgInstaller::new(self.version);
            match installer.install(self.cache_dir.clone()).await {
                Ok(_) => {
                    return Ok(self.cache_dir.clone());
                }
                Err(_) => {
                    return Err(anyhow!(
                        "Failed to install PostgreSQL {}",
                        self.version.as_str()
                    ))
                }
            };
        }
    }

    pub async fn get_psql_path(&self) -> Result<PathBuf> {
        let path = self
            .get_pg_base_path()
            .await?
            .join(format!("{}/bin/psql", self.version.as_str()));

        Ok(path)
    }

    pub async fn get_psql_command(&self) -> Result<Command> {
        let psql_path = self.get_psql_path().await?;
        Ok(Command::new(&psql_path))
    }

    pub async fn get_pg_dump_path(&self) -> Result<PathBuf> {
        let path = self
            .get_pg_base_path()
            .await?
            .join(format!("{}/bin/pg_dump", self.version.as_str()));

        Ok(path)
    }

    pub async fn get_pg_dump_command(&self) -> Result<Command> {
        let pg_dump_path = self.get_pg_dump_path().await?;

        if !std::path::Path::new(&pg_dump_path).exists() {
            return Err(anyhow!(
                "pg_dump executable not found at {}. Make sure PostgreSQL {} is properly installed.",
                pg_dump_path.display(),
                self.version.as_str()
            ));
        }

        Ok(Command::new(&pg_dump_path))
    }

    pub async fn get_pg_restore_path(&self) -> Result<PathBuf> {
        let path = self
            .get_pg_base_path()
            .await?
            .join(format!("{}/bin/pg_restore", self.version.as_str()));

        Ok(path)
    }

    pub async fn get_pg_restore_command(&self) -> Result<Command> {
        let pg_restore_path = self.get_pg_restore_path().await?;

        if !std::path::Path::new(&pg_restore_path).exists() {
            return Err(anyhow!(
                "pg_restore executable not found at {}. Make sure PostgreSQL {} is properly installed.",
                pg_restore_path.display(),
                self.version.as_str()
            ));
        }

        Ok(Command::new(&pg_restore_path))
    }

    pub fn extract_postgres_version(version_string: &str) -> Option<(u32, u32)> {
        // Look for "PostgreSQL X.Y" pattern
        let pg_regex = regex::Regex::new(r"PostgreSQL (\d+)\.(\d+)").ok()?;

        // Try to find and capture the version numbers
        pg_regex.captures(version_string).map(|caps| {
            let major = caps
                .get(1)
                .map(|m| m.as_str().parse::<u32>().unwrap_or(0))
                .unwrap_or(0);
            let minor = caps
                .get(2)
                .map(|m| m.as_str().parse::<u32>().unwrap_or(0))
                .unwrap_or(0);
            (major, minor)
        })
    }

    pub async fn get_postgres_version(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<PostgresVersion> {
        let mut cmd = self.get_psql_command().await?;

        // Add connection parameters
        cmd.arg("-h")
            .arg(host)
            .arg("-p")
            .arg(port.to_string())
            .arg("-U")
            .arg(username)
            .arg("-d")
            .arg(database)
            .arg("-t") // Tuple only output
            .arg("-c")
            .arg("SELECT version();"); // Command to get version

        // If password is provided,set it as an environment variable
        if let Some(pass) = password {
            cmd.env("PGPASSWORD", pass);
        }

        // Execute the command
        let output = cmd
            .output()
            .await
            .context("Failed to execute psql command")?;

        // Check if the command was successful
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("PostgreSQL version check failed: {}", error));
        }

        // Parse the output to get the version
        let string_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let version = match PgTools::extract_postgres_version(&string_version) {
            Some((major, minor)) => {
                let version_str = format!("{}", major);
                match PostgresVersion::from_str(&version_str) {
                    Some(version) => version,
                    None => {
                        return Err(anyhow!(
                            "Unsupported PostgreSQL version {}.{}",
                            major,
                            minor
                        ))
                    }
                }
            }
            None => return Err(anyhow!("Unable to extract PostgreSQL version")),
        };

        Ok(version)
    }

    pub async fn is_postgres_connected(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<bool> {
        let mut cmd = self.get_psql_command().await?;

        // Add connection parameters
        cmd.arg("-h")
            .arg(host)
            .arg("-p")
            .arg(port.to_string())
            .arg("-U")
            .arg(username)
            .arg("-d")
            .arg(database)
            .arg("-c")
            .arg("SELECT 1"); // Simple query to check connection

        // Add timeout to avoid hanging
        cmd.arg("-v")
            .arg("TIMEOUT=5")
            .arg("--set")
            .arg("statement_timeout=5000"); // 5 second timeout

        // If password is provided, set it as an environment variable
        if let Some(pass) = password {
            cmd.env("PGPASSWORD", pass);
        }

        // Execute the command
        let output = match cmd.output().await {
            Ok(output) => output,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to execute psql command: {}. Check if PostgreSQL client is installed.",
                    e
                ));
            }
        };

        // Check if the command was successful
        if output.status.success() {
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            // Don't return an error, just return false with context
            if error.contains("timeout") {
                return Ok(false); // Connection timed out
            } else if error.contains("could not connect") || error.contains("connection to server")
            {
                return Ok(false); // Connection failed but not an error in our function
            }

            // For unexpected errors, return detailed information
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
        compression: Option<u8>,
    ) -> Result<Bytes> {
        // First check that target version matches our version
        let target_version = self
            .get_postgres_version(database, host, port, username, password)
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

        let pg_dump_path = self.get_pg_dump_path().await?;

        if let Some(compression) = compression {
            // Set up pg_dump command
            let mut pg_dump_cmd = self.get_pg_dump_command().await?;
            pg_dump_cmd
                .arg("--format=custom")
                .arg(format!("--host={}", host))
                .arg(format!("--port={}", port))
                .arg(format!("--username={}", username))
                .arg(database)
                .stdout(Stdio::piped());

            if let Some(pass) = password {
                pg_dump_cmd.env("PGPASSWORD", pass);
            }

            // Start pg_dump process
            let mut pg_dump_process = pg_dump_cmd.spawn().context("Failed to execute pg_dump")?;

            // Set up gzip command
            let mut gzip_cmd = Command::new("gzip");
            gzip_cmd
                .arg(format!("-{}", compression))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());

            // Start gzip process
            let mut gzip_process = gzip_cmd.spawn().context("Failed to execute gzip")?;

            // Get handles to the stdout/stdin of both processes
            let pg_dump_stdout = pg_dump_process
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture pg_dump stdout"))?;
            let gzip_stdin = gzip_process
                .stdin
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdin"))?;

            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                // Convert the stdout of pg_dump to a tokio readable
                let pg_dump_stdout = pg_dump_stdout;

                // Convert the stdin of gzip to a tokio writable
                let mut gzip_stdin = gzip_stdin;

                // Copy data from pg_dump to gzip
                let mut buffer = vec![0; 8192]; // 8KB buffer
                let mut reader = tokio::io::BufReader::new(pg_dump_stdout);

                loop {
                    let n = reader
                        .read(&mut buffer)
                        .await
                        .expect("Failed to read from pg_dump");
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
            let (pg_dump_status, output) =
                tokio::join!(pg_dump_process.wait(), gzip_process.wait_with_output());

            let pg_dump_status = pg_dump_status.context("Failed to wait for pg_dump")?;
            let output = output.context("Failed to read gzip output")?;

            if !pg_dump_status.success() {
                let exit_code = pg_dump_status.code().unwrap_or(-1);
                return Err(anyhow!("pg_dump failed with exit code: {}", exit_code));
            }

            // Check gzip process exit status
            if !output.status.success() {
                let exit_code = output.status.code().unwrap_or(-1);
                return Err(anyhow!("gzip failed with exit code: {}", exit_code));
            }

            Ok(Bytes::from(output.stdout))
        } else {
            let mut cmd = self.get_pg_dump_command().await?;
            cmd.arg("--format=custom")
                .arg("--compress=9") // PostgreSQL's internal compression
                .arg(format!("--host={}", host))
                .arg(format!("--port={}", port))
                .arg(format!("--username={}", username))
                .arg(database)
                .stdout(Stdio::piped()); // Ensure stdout is captured

            // Set the PGPASSWORD environment variable if password is provided
            if let Some(pass) = password {
                cmd.env("PGPASSWORD", pass);
            }

            // Execute the command
            let output = cmd.output().await.context(format!(
                "Failed to execute pg_dump command at {}",
                pg_dump_path.display()
            ))?;

            // Check if the command executed successfully
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                // Provide specific error messages based on common failure patterns
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
                        "pg_dump failed with exit code {}.\nCommand: {} --format=custom --compress=9 --host={} --port={} --username={} {}\n\nError details: {}",
                        exit_code,
                        pg_dump_path.display(),
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
                return Err(anyhow!("pg_dump completed but didn't produce any output. This might indicate a problem with the dump process."));
            }

            Ok(Bytes::from(output.stdout))
        }
    }

    pub async fn restore(
        &self,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
        dump_data: Bytes,
        compressed: bool,
    ) -> Result<()> {
        let target_version = self
            .get_postgres_version(database, host, port, username, password)
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

        let pg_restore_path = self.get_pg_restore_path().await?;

        if compressed {
            // Create a temporary file to store the compressed data
            let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
            let compressed_file_path = temp_dir.path().join("dump.gz");
            let decompressed_file_path = temp_dir.path().join("dump.pgdump");

            // Write the compressed data to a temporary file
            tokio::fs::write(&compressed_file_path, &dump_data)
                .await
                .context("Failed to write compressed data to temporary file")?;

            // Set up gunzip command to decompress the data
            let mut gunzip_cmd = Command::new("gunzip");
            gunzip_cmd
                .arg("-c")
                .arg(&compressed_file_path)
                .stdout(Stdio::piped());

            // Start gunzip process
            let mut gunzip_process = gunzip_cmd.spawn().context("Failed to execute gunzip")?;

            // Get handle to gunzip stdout
            let mut gunzip_stdout = gunzip_process
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture gunzip stdout"))?;

            // Write the decompressed data to a temporary file
            let mut decompressed_file = tokio::fs::File::create(&decompressed_file_path)
                .await
                .context("Failed to create decompressed file")?;

            // Copy data from gunzip to the temporary file
            tokio::io::copy(&mut gunzip_stdout, &mut decompressed_file)
                .await
                .context("Failed to decompress data")?;

            // Wait for gunzip to complete
            let gunzip_status = gunzip_process
                .wait()
                .await
                .context("Failed to wait for gunzip")?;
            if !gunzip_status.success() {
                let exit_code = gunzip_status.code().unwrap_or(-1);
                return Err(anyhow!("gunzip failed with exit code: {}", exit_code));
            }

            // Set up pg_restore command
            let mut pg_restore_cmd = self.get_pg_restore_command().await?;
            pg_restore_cmd
                .arg(format!("--host={}", host))
                .arg(format!("--port={}", port))
                .arg(format!("--username={}", username))
                .arg("--dbname=".to_string() + database)
                .arg("--no-owner")
                .arg("--no-privileges")
                .arg("--clean") // Drop objects before recreating them
                .arg("--if-exists") // Add IF EXISTS for cleaner drops
                .arg("--no-comments") // Skip comments to avoid warnings
                .arg("--exit-on-error") // Continue on error
                .arg(&decompressed_file_path);

            // Set the PGPASSWORD environment variable if password is provided
            if let Some(pass) = password {
                pg_restore_cmd.env("PGPASSWORD", pass);
            }

            // Execute the command
            let output = pg_restore_cmd.output().await.context(format!(
                "Failed to execute pg_restore command at {}",
                pg_restore_path.display()
            ))?;

            // Check if the command executed successfully
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                // Provide specific error messages based on common failure patterns
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
                        "pg_restore failed with exit code {}.\nCommand: {} --host={} --port={} --username={} --dbname={} --no-owner --no-privileges {}\n\nError details: {}",
                        exit_code,
                        pg_restore_path.display(),
                        host,
                        port,
                        username,
                        database,
                        decompressed_file_path.display(),
                        stderr.trim()
                    ));
                }
            }

            Ok(())
        } else {
            // Create a temporary file to store the dump data
            let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
            let dump_file_path = temp_dir.path().join("dump.pgdump");

            // Write the dump data to a temporary file
            tokio::fs::write(&dump_file_path, &dump_data)
                .await
                .context("Failed to write dump data to temporary file")?;

            // Set up pg_restore command
            let mut cmd = self.get_pg_restore_command().await?;
            cmd.arg(format!("--host={}", host))
                .arg(format!("--port={}", port))
                .arg(format!("--username={}", username))
                .arg("--dbname=".to_string() + database)
                .arg("--no-owner")
                .arg("--no-privileges")
                .arg("--clean") // Drop objects before recreating them
                .arg("--if-exists") // Add IF EXISTS for cleaner drops
                .arg("--no-comments") // Skip comments to avoid warnings
                .arg("--exit-on-error") // Continue on error
                .arg(&dump_file_path);

            // Set the PGPASSWORD environment variable if password is provided
            if let Some(pass) = password {
                cmd.env("PGPASSWORD", pass);
            }

            // Execute the command
            let output = cmd.output().await.context(format!(
                "Failed to execute pg_restore command at {}",
                pg_restore_path.display()
            ))?;

            // Check if the command executed successfully
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                // Provide specific error messages based on common failure patterns
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
                        "pg_restore failed with exit code {}.\nCommand: {} --host={} --port={} --username={} --dbname={} --no-owner --no-privileges {}\n\nError details: {}",
                        exit_code,
                        pg_restore_path.display(),
                        host,
                        port,
                        username,
                        database,
                        dump_file_path.display(),
                        stderr.trim()
                    ));
                }
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::*;

    fn initialize_test() {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .init();
    }

    #[test]
    fn test_pg_tools_initialization() {
        initialize_test();
        let pg_tools = PgTools::default().expect("Failed to initialize PgTools");
        assert!(pg_tools.cache_dir.exists());
        assert_eq!(pg_tools.version, PostgresVersion::V15);
    }

    #[tokio::test]
    async fn test_pg_connection() {
        initialize_test();
        let pg_tools = PgTools::default().expect("Failed to initialize PgTools");
        let is_connected = pg_tools
            .is_postgres_connected("api", "localhost", 5432, "postgres", Some("postgres"))
            .await
            .expect("Unable to check PostgreSQL connection");

        assert!(is_connected);
    }

    #[tokio::test]
    async fn test_pg_tool_detect_version() {
        initialize_test();
        let pg_tools = PgTools::default().expect("Failed to initialize PgTools");
        let version = pg_tools
            .get_postgres_version("api", "localhost", 5432, "postgres", Some("postgres"))
            .await
            .expect("Failed to get postgres version");

        assert_eq!(version, PostgresVersion::V17);
    }

    #[tokio::test]
    async fn test_pg_dump() {
        initialize_test();
        let mut pg_tools = PgTools::default().expect("Failed to initialize PgTools");

        let version = pg_tools
            .get_postgres_version("api", "localhost", 5432, "postgres", Some("postgres"))
            .await
            .expect("Failed to get postgres version");

        pg_tools = PgTools::new(version).expect("Failed to initialize PgTools");

        let bytes = pg_tools
            .dump("api", "localhost", 5432, "postgres", Some("postgres"), None)
            .await
            .expect("Unable to dump database");

        let compressed_bytes = pg_tools
            .dump(
                "api",
                "localhost",
                5432,
                "postgres",
                Some("postgres"),
                Some(9),
            )
            .await
            .expect("Unable to dump database");

        assert!(bytes.len() > 0);
        assert!(compressed_bytes.len() > 0);
        assert!(compressed_bytes.len() < bytes.len());
    }
}
