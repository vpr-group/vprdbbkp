use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use crate::databases::mysql::mysql_docker::{get_mysql_version, is_docker_available};

/// Restore a MySQL database from a gzipped backup
pub async fn restore_mysql(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    backup_data: &[u8],
) -> Result<()> {
    info!("Starting MySQL restore for database: {}", database);

    // Create a temporary file for the compressed backup
    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
    temp_file
        .write_all(backup_data)
        .context("Failed to write backup data to temporary file")?;
    temp_file
        .flush()
        .context("Failed to flush temporary file")?;

    // Command to decompress the backup
    let mut gunzip_cmd = Command::new("gunzip");
    gunzip_cmd
        .arg("-c")
        .arg(temp_file.path())
        .stdout(Stdio::piped());

    // Command to restore using mysql client
    let mut mysql_cmd = Command::new("mysql");
    mysql_cmd
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--user={}", username))
        .arg(database)
        .stdin(Stdio::piped());

    if let Some(password) = password {
        mysql_cmd.arg(format!("--password={}", password));
    }

    // Execute gunzip
    debug!("Executing gunzip command");
    let mut gunzip_process = gunzip_cmd.spawn().context("Failed to execute gunzip")?;

    // Execute mysql
    debug!("Executing mysql command");
    let mut mysql_process = mysql_cmd.spawn().context("Failed to execute mysql")?;

    // Connect the processes: gunzip output -> mysql input
    let gunzip_stdout = gunzip_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gunzip stdout"))?;

    let mut mysql_stdin = mysql_process
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture mysql stdin"))?;

    // Pipe gunzip stdout to mysql stdin
    tokio::task::spawn_blocking(move || {
        std::io::copy(
            &mut std::io::BufReader::new(gunzip_stdout),
            &mut mysql_stdin,
        )
        .context("Failed to pipe gunzip output to mysql")
    });

    // Wait for processes to complete
    let gunzip_status = gunzip_process.wait().context("Failed to wait for gunzip")?;
    if !gunzip_status.success() {
        error!("gunzip failed with exit code: {:?}", gunzip_status.code());
        return Err(anyhow::anyhow!("gunzip command failed"));
    }

    let mysql_status = mysql_process.wait().context("Failed to wait for mysql")?;
    if !mysql_status.success() {
        error!("mysql failed with exit code: {:?}", mysql_status.code());
        return Err(anyhow::anyhow!("mysql command failed"));
    }

    info!("Database restore completed successfully");
    Ok(())
}

/// Restore a MySQL database from a gzipped backup using Docker
pub async fn restore_mysql_with_docker(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    backup_data: &[u8],
    mysql_version: (u32, u32),
) -> Result<()> {
    info!(
        "Starting MySQL restore using Docker for database: {}",
        database
    );

    // Create a temporary file for the compressed backup
    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
    temp_file
        .write_all(backup_data)
        .context("Failed to write backup data to temporary file")?;
    temp_file
        .flush()
        .context("Failed to flush temporary file")?;

    let (major, minor) = mysql_version;
    let version_tag = format!("{}.{}", major, minor);
    let mysql_image = format!("mysql:{}", version_tag);

    // Command to decompress the backup
    let mut gunzip_cmd = Command::new("gunzip");
    gunzip_cmd
        .arg("-c")
        .arg(temp_file.path())
        .stdout(Stdio::piped());

    // Command to restore using Docker
    let mut docker_cmd = Command::new("docker");
    docker_cmd
        .arg("run")
        .arg("--rm")
        .arg("--network=host") // Use host network to connect to the MySQL server
        .arg("-i") // Interactive to allow piping
        .arg(&mysql_image)
        .arg("mysql")
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--user={}", username))
        .arg(database)
        .stdin(Stdio::piped());

    if let Some(password) = password {
        docker_cmd.arg(format!("--password={}", password));
    }

    // Execute gunzip
    debug!("Executing gunzip command");
    let mut gunzip_process = gunzip_cmd.spawn().context("Failed to execute gunzip")?;

    // Execute Docker
    debug!("Executing Docker mysql command");
    let mut docker_process = docker_cmd
        .spawn()
        .context("Failed to execute Docker command")?;

    // Connect the processes: gunzip output -> docker input
    let gunzip_stdout = gunzip_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gunzip stdout"))?;

    let mut docker_stdin = docker_process
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture Docker stdin"))?;

    // Pipe gunzip stdout to docker stdin
    tokio::task::spawn_blocking(move || {
        std::io::copy(
            &mut std::io::BufReader::new(gunzip_stdout),
            &mut docker_stdin,
        )
        .context("Failed to pipe gunzip output to Docker")
    });

    // Wait for processes to complete
    let gunzip_status = gunzip_process.wait().context("Failed to wait for gunzip")?;
    if !gunzip_status.success() {
        error!("gunzip failed with exit code: {:?}", gunzip_status.code());
        return Err(anyhow::anyhow!("gunzip command failed"));
    }

    let docker_status = docker_process.wait().context("Failed to wait for Docker")?;
    if !docker_status.success() {
        error!(
            "Docker mysql failed with exit code: {:?}",
            docker_status.code()
        );
        return Err(anyhow::anyhow!("Docker mysql command failed"));
    }

    info!("Database restore with Docker completed successfully");
    Ok(())
}

/// Main restore function with Docker fallback option
pub async fn restore_mysql_with_options(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    backup_data: &[u8],
    force_docker: bool,
) -> Result<()> {
    // Check MySQL server version
    let server_version = get_mysql_version(database, host, port, username, password).await?;
    debug!("Detected MySQL server version: {:?}", server_version);

    if force_docker {
        info!("Forcing Docker usage for restore as requested");
        if is_docker_available() {
            return restore_mysql_with_docker(
                database,
                host,
                port,
                username,
                password,
                backup_data,
                server_version,
            )
            .await;
        } else {
            warn!("Docker was requested but is not available");
            warn!("Falling back to local mysql client");
        }
    }

    // Use local mysql client
    restore_mysql(database, host, port, username, password, backup_data).await
}
