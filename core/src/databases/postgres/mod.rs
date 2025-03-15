use anyhow::{Context, Result};
use bytes::Bytes;
use log::{debug, error, info, warn};
use pg_tools::PgTools;
use std::process::{Command, Stdio};

pub mod pg_docker;
pub mod pg_restore;
pub mod pg_tools;
pub mod pg_versions;

pub async fn is_postgres_connected(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
) -> Result<bool> {
    let pg_tools = PgTools::default()?;

    let is_connected = pg_tools
        .is_postgres_connected(database, host, port, username, password)
        .await?;

    Ok(is_connected)
}

pub async fn backup_postgres(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: Option<u8>,
) -> Result<Bytes> {
    let mut pg_tools = PgTools::default()?;

    let version = pg_tools
        .get_postgres_version(database, host, port, username, password)
        .await?;

    // Set the correct PostrgreSQL target version
    pg_tools = PgTools::new(version)?;

    let output = pg_tools
        .dump(database, host, port, username, password, compression)
        .await?;

    Ok(output)
}

pub async fn restore_postgres(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    dump_data: Bytes,
    compressed: bool,
) -> Result<()> {
    let mut pg_tools = PgTools::default()?;

    let version = pg_tools
        .get_postgres_version(database, host, port, username, password)
        .await?;

    // Set the correct PostrgreSQL target version
    pg_tools = PgTools::new(version)?;

    pg_tools
        .restore(
            database, host, port, username, password, dump_data, compressed,
        )
        .await?;

    Ok(())
}

/// Get the local pg_dump version
fn get_local_pg_dump_version() -> Result<(u32, u32)> {
    let output = Command::new("pg_dump")
        .arg("--version")
        .output()
        .context("Failed to execute pg_dump to get version")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get pg_dump version"));
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    parse_pg_dump_version(&version_str)
}

/// Parse pg_dump version output to extract major and minor version
fn parse_pg_dump_version(version_output: &str) -> Result<(u32, u32)> {
    // Example format: "pg_dump (PostgreSQL) 14.8 (Ubuntu 14.8-0ubuntu0.22.04.1)"
    let re = regex::Regex::new(r"pg_dump \(PostgreSQL\) (\d+)(?:\.(\d+))?")
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
        "Failed to parse pg_dump version: {}",
        version_output
    ))
}

/// Check if pg_dump version is compatible with server version
fn is_pg_dump_compatible(pg_dump_version: (u32, u32), server_version: (u32, u32)) -> bool {
    let (pg_dump_major, _) = pg_dump_version;
    let (server_major, _) = server_version;

    // PostgreSQL requires pg_dump to be the same or newer major version
    pg_dump_major >= server_major
}

// Add a new public function that includes the force_docker option
pub async fn backup_postgres_with_options(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: u8,
    force_docker: bool,
) -> Result<Bytes> {
    info!("Starting PostgreSQL backup for database: {}", database);

    // Check PostgreSQL version compatibility
    let server_version =
        pg_docker::get_postgres_version(database, host, port, username, password).await?;
    debug!("Detected PostgreSQL server version: {:?}", server_version);

    // Get local pg_dump version
    let local_version = get_local_pg_dump_version()?;
    debug!("Local pg_dump version: {:?}", local_version);

    // Check if versions are compatible
    let versions_compatible = is_pg_dump_compatible(local_version, server_version);

    // Use Docker if versions are incompatible or if force_docker is true
    if !versions_compatible || force_docker {
        if !versions_compatible {
            info!(
                "PostgreSQL version mismatch detected: server {:?}, local pg_dump {:?}",
                server_version, local_version
            );
        }

        if force_docker {
            info!("Forcing Docker usage as requested");
        }

        if pg_docker::is_docker_available() {
            info!("Using Docker with matching PostgreSQL version for backup");
            return pg_docker::docker_pg_dump(
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
                warn!("PostgreSQL version mismatch detected but Docker is not available");
                warn!("Attempting backup with local pg_dump, but it might fail");
            }
        }
    }

    // Use local pg_dump (if versions are compatible or Docker isn't available)
    info!("Using local pg_dump for backup");
    let mut pg_dump_cmd = Command::new("pg_dump");
    pg_dump_cmd
        .arg("--format=custom")
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--username={}", username))
        .arg(database)
        .stdout(Stdio::piped());

    if let Some(password) = password {
        pg_dump_cmd.env("PGPASSWORD", password);
    }

    let mut gzip_cmd = Command::new("gzip");
    gzip_cmd
        .arg(format!("-{}", compression))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    debug!("Executing pg_dump command");
    let mut pg_dump_process = pg_dump_cmd.spawn().context("Failed to execute pg_dump")?;

    debug!("Executing gzip command");
    let mut gzip_process = gzip_cmd.spawn().context("Failed to execute gzip")?;

    let pg_dump_stdout = pg_dump_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture pg_dump stdout"))?;
    let mut gzip_stdin = gzip_process
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdin"))?;

    // Pipe pg_dump stdout to gzip stdin
    tokio::task::spawn_blocking(move || {
        std::io::copy(
            &mut std::io::BufReader::new(pg_dump_stdout),
            &mut gzip_stdin,
        )
        .context("Failed to pipe pg_dump output to gzip")
    });

    // Read compressed output
    let mut output = Vec::new();
    let mut gzip_stdout = gzip_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdout"))?;
    std::io::Read::read_to_end(&mut gzip_stdout, &mut output)?;

    // Check process exit statuses
    let pg_dump_status = pg_dump_process
        .wait()
        .context("Failed to wait for pg_dump")?;
    if !pg_dump_status.success() {
        error!("pg_dump failed with exit code: {:?}", pg_dump_status.code());
        return Err(anyhow::anyhow!("pg_dump command failed"));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection() {
        let is_connected =
            is_postgres_connected("api", "localhost", 5432, "postgres", Some("postgres"))
                .await
                .expect("Unable to check database connection");

        let is_not_connected =
            is_postgres_connected("random", "localhost", 5432, "postgres", Some("postgres"))
                .await
                .expect("Unable to check database connection");

        assert!(is_connected);
        assert_eq!(is_not_connected, false);
    }

    #[tokio::test]
    async fn test_backup() {
        let backup = backup_postgres("api", "localhost", 5432, "postgres", Some("postgres"), None)
            .await
            .expect("Unable to backup database");

        let compressed_backup = backup_postgres(
            "api",
            "localhost",
            5432,
            "postgres",
            Some("postgres"),
            Some(9),
        )
        .await
        .expect("Unable to backup database");

        assert!(backup.len() > 0);
        assert!(compressed_backup.len() > 0);
        assert!(compressed_backup.len() < backup.len());
    }
}
