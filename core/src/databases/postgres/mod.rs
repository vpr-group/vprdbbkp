use anyhow::{Context, Result};
use bytes::Bytes;
use log::{debug, error, info, warn};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::process::{Command, Stdio};
use tokio_postgres::NoTls;

pub mod pg_docker;
pub mod pg_restore;
pub mod pg_tools;
pub mod pg_versions;

use std::time::Duration;

pub async fn is_postgres_connected(
    host: &str,
    port: u16,
    database: &str,
    username: &str,
    password: Option<&str>,
    timeout_seconds: u64,
    use_ssl: Option<bool>,
) -> Result<bool> {
    info!(
        "Checking PostgreSQL connection to {}:{}/{}",
        host, port, database
    );

    // Build base connection string
    let base_connection_string = match password {
        Some(pass) => format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database, username, pass
        ),
        None => format!(
            "host={} port={} dbname={} user={}",
            host, port, database, username
        ),
    };

    // Set up connection timeout
    let timeout = Duration::from_secs(timeout_seconds);

    // Try with SSL first if use_ssl is None or true
    if use_ssl.unwrap_or(true) {
        debug!("Attempting PostgreSQL connection with SSL");

        // Add sslmode=require to connection string
        let ssl_connection_string = format!("{} sslmode=require", base_connection_string);

        // Configure SSL with OpenSSL
        let mut builder = SslConnector::builder(SslMethod::tls())?;
        builder.set_verify(SslVerifyMode::NONE); // For testing - enable proper verification in production
        let connector = MakeTlsConnector::new(builder.build());

        // Attempt SSL connection
        match tokio::time::timeout(
            timeout.clone(),
            tokio_postgres::connect(&ssl_connection_string, connector),
        )
        .await
        {
            Ok(Ok((client, connection))) => {
                // Spawn the connection handler
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("PostgreSQL SSL connection error: {}", e);
                    }
                });

                // Execute a simple query to validate the connection
                match client.execute("SELECT 1", &[]).await {
                    Ok(_) => {
                        info!("Successfully connected to PostgreSQL database with SSL");
                        return Ok(true);
                    }
                    Err(e) => {
                        error!("Failed to execute test query with SSL: {}", e);
                        if use_ssl == Some(true) {
                            return Ok(false);
                        }
                        // If SSL was just an attempt, we continue to non-SSL below
                        warn!("SSL connection failed, trying without SSL");
                    }
                }
            }
            Ok(Err(e)) => {
                error!("PostgreSQL SSL connection error: {}", e);
                if use_ssl == Some(true) {
                    return Ok(false);
                }
                // If SSL was just an attempt, continue to non-SSL
                warn!("SSL connection failed, trying without SSL");
            }
            Err(_) => {
                error!(
                    "PostgreSQL SSL connection timed out after {} seconds",
                    timeout_seconds
                );
                if use_ssl == Some(true) {
                    return Ok(false);
                }
                // If SSL was just an attempt, continue to non-SSL
                warn!("SSL connection timed out, trying without SSL");
            }
        }
    }

    // Try non-SSL connection if use_ssl is None or false
    if use_ssl.unwrap_or(false) == false {
        debug!("Attempting PostgreSQL connection without SSL");

        // Attempt to connect with NoTls
        match tokio::time::timeout(
            timeout,
            tokio_postgres::connect(&base_connection_string, NoTls),
        )
        .await
        {
            Ok(Ok((client, connection))) => {
                // Spawn the connection handler
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("PostgreSQL connection error: {}", e);
                    }
                });

                // Execute a simple query to validate the connection
                match client.execute("SELECT 1", &[]).await {
                    Ok(_) => {
                        info!("Successfully connected to PostgreSQL database without SSL");
                        return Ok(true);
                    }
                    Err(e) => {
                        error!("Failed to execute test query without SSL: {}", e);
                        return Ok(false);
                    }
                }
            }
            Ok(Err(e)) => {
                error!("PostgreSQL connection error without SSL: {}", e);
                return Ok(false);
            }
            Err(_) => {
                error!(
                    "PostgreSQL connection timed out after {} seconds (without SSL)",
                    timeout_seconds
                );
                return Ok(false);
            }
        }
    }

    // If we reach here, both SSL and non-SSL failed
    warn!("All PostgreSQL connection attempts failed");
    Ok(false)
}

pub async fn is_postgres_connected_default_timeout(
    host: &str,
    port: u16,
    database: &str,
    username: &str,
    password: Option<&str>,
) -> Result<bool> {
    is_postgres_connected(host, port, database, username, password, 5, None).await
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

// Keep the original function for backward compatibility
pub async fn backup_postgres(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: u8,
) -> Result<Bytes> {
    backup_postgres_with_options(database, host, port, username, password, compression, false).await
}
