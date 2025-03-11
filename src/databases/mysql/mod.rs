use anyhow::{Context, Result};
use bytes::Bytes;
use log::{debug, error, info, warn};
use std::process::{Command, Stdio};

pub mod mysql_docker;
pub mod mysql_restore;

/// Get the local mysqldump version
fn get_local_mysqldump_version() -> Result<(u32, u32)> {
    let output = Command::new("mysqldump")
        .arg("--version")
        .output()
        .context("Failed to execute mysqldump to get version")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get mysqldump version"));
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    parse_mysqldump_version(&version_str)
}

/// Parse mysqldump version output to extract major and minor version
fn parse_mysqldump_version(version_output: &str) -> Result<(u32, u32)> {
    // Example format: "mysqldump  Ver 8.0.32-0ubuntu0.22.04.2 for Linux on x86_64"
    let re = regex::Regex::new(r"mysqldump\s+Ver\s+(\d+)(?:\.(\d+))?")
        .context("Failed to compile regex")?;

    if let Some(caps) = re.captures(version_output) {
        let major: u32 = caps
            .get(1)
            .map(|m| m.as_str().parse())
            .context("Failed to parse major version")?
            .context("Failed to parse major version as u32")?;

        let minor: u32 = caps
            .get(2)
            .map(|m| m.as_str().parse().unwrap_or(0))
            .unwrap_or(0);

        return Ok((major, minor));
    }

    Err(anyhow::anyhow!(
        "Failed to parse mysqldump version: {}",
        version_output
    ))
}

/// Check if mysqldump version is compatible with server version
fn is_mysqldump_compatible(mysqldump_version: (u32, u32), server_version: (u32, u32)) -> bool {
    let (mysqldump_major, _mysqldump_minor) = mysqldump_version;
    let (server_major, _server_minor) = server_version;

    // MySQL is more flexible than PostgreSQL, but it's generally best to use
    // a mysqldump version that's the same or newer than the server
    // However, using a much newer client can sometimes cause issues
    if mysqldump_major > server_major {
        // Allow one major version ahead
        mysqldump_major - server_major <= 1
    } else if mysqldump_major == server_major {
        // Same major version is always good
        true
    } else {
        // Older major version is generally not compatible
        false
    }
}

// Main backup function with force_docker option
pub async fn backup_mysql_with_options(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: u8,
    force_docker: bool,
) -> Result<Bytes> {
    info!("Starting MySQL backup for database: {}", database);

    // Check MySQL server version
    let server_version =
        mysql_docker::get_mysql_version(database, host, port, username, password).await?;
    debug!("Detected MySQL server version: {:?}", server_version);

    // Get local mysqldump version
    let local_version = get_local_mysqldump_version()?;
    debug!("Local mysqldump version: {:?}", local_version);

    // Check if versions are compatible
    let versions_compatible = is_mysqldump_compatible(local_version, server_version);

    // Use Docker if versions are incompatible or if force_docker is true
    if !versions_compatible || force_docker {
        if !versions_compatible {
            info!(
                "MySQL version mismatch detected: server {:?}, local mysqldump {:?}",
                server_version, local_version
            );
        }

        if force_docker {
            info!("Forcing Docker usage as requested");
        }

        if mysql_docker::is_docker_available() {
            info!("Using Docker with matching MySQL version for backup");
            return mysql_docker::docker_mysqldump(
                database,
                host,
                port,
                username,
                password,
                compression,
                server_version,
            )
            .await
            .map(Bytes::from);
        } else {
            warn!("Docker was requested but is not available");
            if !versions_compatible {
                warn!("MySQL version mismatch detected but Docker is not available");
                warn!("Attempting backup with local mysqldump, but it might fail");
            }
        }
    }

    // Use local mysqldump (if versions are compatible or Docker isn't available)
    info!("Using local mysqldump for backup");
    let mut mysqldump_cmd = Command::new("mysqldump");
    mysqldump_cmd
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--user={}", username))
        .arg("--protocol=tcp") // Force TCP protocol
        .arg("--single-transaction") // Consistent snapshot without locking tables
        .arg("--routines") // Include stored procedures and functions
        .arg("--triggers") // Include triggers
        .arg("--events") // Include events
        .arg("--set-gtid-purged=OFF") // Avoid GTID information
        .arg("--no-tablespaces") // Avoid accessing TABLESPACES
        .arg("--column-statistics=0") // Disable column statistics
        .arg(database)
        .stdout(Stdio::piped());

    if let Some(password) = password {
        mysqldump_cmd.arg(format!("--password={}", password));
    }

    let mut gzip_cmd = Command::new("gzip");
    gzip_cmd
        .arg(format!("-{}", compression))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    debug!("Executing mysqldump command");
    let mut mysqldump_process = mysqldump_cmd
        .spawn()
        .context("Failed to execute mysqldump")?;

    debug!("Executing gzip command");
    let mut gzip_process = gzip_cmd.spawn().context("Failed to execute gzip")?;

    let mysqldump_stdout = mysqldump_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture mysqldump stdout"))?;
    let mut gzip_stdin = gzip_process
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdin"))?;

    // Pipe mysqldump stdout to gzip stdin
    tokio::task::spawn_blocking(move || {
        std::io::copy(
            &mut std::io::BufReader::new(mysqldump_stdout),
            &mut gzip_stdin,
        )
        .context("Failed to pipe mysqldump output to gzip")
    });

    // Read compressed output
    let mut output = Vec::new();
    let mut gzip_stdout = gzip_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdout"))?;
    std::io::Read::read_to_end(&mut gzip_stdout, &mut output)?;

    // Check process exit statuses
    let mysqldump_status = mysqldump_process
        .wait()
        .context("Failed to wait for mysqldump")?;
    if !mysqldump_status.success() {
        error!(
            "mysqldump failed with exit code: {:?}",
            mysqldump_status.code()
        );
        return Err(anyhow::anyhow!("mysqldump command failed"));
    }

    let gzip_status = gzip_process.wait().context("Failed to wait for gzip")?;
    if !gzip_status.success() {
        error!("gzip failed with exit code: {:?}", gzip_status.code());
        return Err(anyhow::anyhow!("gzip command failed"));
    }

    info!(
        "Backup completed successfully, size: {} bytes",
        output.len()
    );
    Ok(Bytes::from(output))
}

// Backward compatibility function
pub async fn backup_mysql(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: u8,
) -> Result<Bytes> {
    backup_mysql_with_options(database, host, port, username, password, compression, false).await
}
