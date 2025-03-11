use anyhow::{Context, Result};
use bytes::Bytes;
use flate2::read::GzDecoder;
use log::{debug, error, info, warn};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use crate::databases::postgres::pg_docker::get_postgres_version;

/// Restore a PostgreSQL database from a compressed backup
pub async fn restore_postgres(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compressed_backup: Bytes,
    force_docker: bool,
    drop_db: bool,
) -> Result<()> {
    info!("Starting PostgreSQL restore for database: {}", database);

    // Check PostgreSQL version compatibility
    let server_version = match get_postgres_version(database, host, port, username, password).await
    {
        Ok(version) => version,
        Err(_) => {
            // If database doesn't exist yet, try to connect to 'postgres' default DB
            warn!(
                "Could not connect to database '{}', trying 'postgres' instead",
                database
            );
            get_postgres_version("postgres", host, port, username, password).await?
        }
    };

    debug!("Detected PostgreSQL server version: {:?}", server_version);

    // Get decompressed backup data
    let backup_data = decompress_backup(compressed_backup)?;
    info!("Decompressed backup size: {} bytes", backup_data.len());

    // Save to temp file
    let backup_file = create_temp_file(&backup_data)?;
    let backup_path = backup_file.path().to_string_lossy().to_string();
    debug!("Backup saved to temporary file: {}", backup_path);

    // Check if we need to create the database
    ensure_database_exists(database, host, port, username, password, drop_db).await?;

    // Restore using local pg_restore or Docker
    let restore_result = if force_docker {
        info!("Forcing Docker for restore as requested");
        restore_pg_docker(
            database,
            host,
            port,
            username,
            password,
            &backup_path,
            server_version,
        )
        .await
    } else {
        info!("Using local pg_restore for restore");
        restore_pg_local(database, host, port, username, password, &backup_path).await
    };

    // Clean up the temp file (it will be removed automatically when the variable goes out of scope,
    // but let's be explicit for clarity)
    std::fs::remove_file(&backup_path).ok(); // Ignore errors on cleanup

    restore_result
}

/// Create a database if it doesn't exist, optionally dropping it first
async fn ensure_database_exists(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    drop_first: bool,
) -> Result<()> {
    // Connect to the 'postgres' database to check/create the target database
    let mut psql_cmd = Command::new("psql");
    psql_cmd
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--username={}", username))
        .arg("--dbname=postgres")
        .arg("--tuples-only")
        .arg("--no-align")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(pass) = password {
        psql_cmd.env("PGPASSWORD", pass);
    }

    // Check if database exists
    let check_cmd = format!("SELECT 1 FROM pg_database WHERE datname = '{}'", database);
    psql_cmd.arg(format!("--command={}", check_cmd));

    debug!("Checking if database exists: {}", database);
    let output = psql_cmd
        .output()
        .context("Failed to execute psql command")?;

    let mut exists = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim() == "1"
    } else {
        error!(
            "Database check failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(anyhow::anyhow!("Failed to check if database exists"));
    };

    // Drop database if requested and it exists
    if drop_first && exists {
        info!("Dropping existing database as requested: {}", database);

        let mut drop_cmd = Command::new("psql");
        drop_cmd
            .arg(format!("--host={}", host))
            .arg(format!("--port={}", port))
            .arg(format!("--username={}", username))
            .arg("--dbname=postgres")
            .arg(format!("--command=DROP DATABASE \"{}\"", database))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(pass) = password {
            drop_cmd.env("PGPASSWORD", pass);
        }

        let drop_output = drop_cmd
            .output()
            .context("Failed to execute DROP DATABASE command")?;

        if !drop_output.status.success() {
            error!(
                "Failed to drop database: {}",
                String::from_utf8_lossy(&drop_output.stderr)
            );
            return Err(anyhow::anyhow!("Failed to drop database"));
        }

        // Database no longer exists
        info!("Database dropped successfully");
        exists = false;
    }

    // Create database if it doesn't exist
    if !exists {
        info!("Creating database: {}", database);

        let mut create_cmd = Command::new("psql");
        create_cmd
            .arg(format!("--host={}", host))
            .arg(format!("--port={}", port))
            .arg(format!("--username={}", username))
            .arg("--dbname=postgres")
            .arg(format!("--command=CREATE DATABASE \"{}\"", database))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(pass) = password {
            create_cmd.env("PGPASSWORD", pass);
        }

        let create_output = create_cmd
            .output()
            .context("Failed to execute CREATE DATABASE command")?;

        if !create_output.status.success() {
            error!(
                "Failed to create database: {}",
                String::from_utf8_lossy(&create_output.stderr)
            );
            return Err(anyhow::anyhow!("Failed to create database"));
        }

        info!("Database created successfully");
    } else {
        info!("Database already exists: {}", database);
    }

    Ok(())
}

/// Decompress a gzipped backup
fn decompress_backup(compressed_data: Bytes) -> Result<Vec<u8>> {
    debug!("Decompressing backup data");

    let mut decoder = GzDecoder::new(&compressed_data[..]);
    let mut decompressed_data = Vec::new();
    std::io::copy(&mut decoder, &mut decompressed_data)?;

    Ok(decompressed_data)
}

/// Save backup data to a temporary file
fn create_temp_file(data: &[u8]) -> Result<NamedTempFile> {
    debug!("Creating temporary file for backup");

    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
    temp_file
        .write_all(data)
        .context("Failed to write to temporary file")?;
    temp_file
        .flush()
        .context("Failed to flush temporary file")?;

    Ok(temp_file)
}

/// Restore PostgreSQL backup using local pg_restore
async fn restore_pg_local(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    backup_path: &str,
) -> Result<()> {
    info!("Restoring PostgreSQL database using local pg_restore");

    let mut pg_restore_cmd = Command::new("pg_restore");
    pg_restore_cmd
        .arg("--dbname")
        .arg(database)
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--username={}", username))
        .arg("--verbose")
        .arg("--no-owner")
        .arg("--no-privileges")
        .arg(backup_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(pass) = password {
        pg_restore_cmd.env("PGPASSWORD", pass);
    }

    debug!("Executing pg_restore command");
    let output = pg_restore_cmd
        .output()
        .context("Failed to execute pg_restore command")?;

    // pg_restore can return non-zero even for successful restores with warnings
    if !output.status.success() {
        // Log the error but don't fail completely if there's output on stdout indicating progress
        warn!(
            "pg_restore returned non-zero exit code: {:?}",
            output.status.code()
        );
        warn!(
            "pg_restore stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // If we see serious errors, fail the restore
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("ERROR:") && !stderr.contains("WARNING:") {
            return Err(anyhow::anyhow!("pg_restore failed with errors"));
        }
    }

    info!("Database restore completed");
    Ok(())
}

/// Restore PostgreSQL backup using Docker
async fn restore_pg_docker(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    backup_path: &str,
    pg_version: (u32, u32),
) -> Result<()> {
    let (major, minor) = pg_version;
    let version_tag = if minor == 0 {
        format!("{}", major)
    } else {
        format!("{}.{}", major, minor)
    };

    info!(
        "Restoring PostgreSQL database using Docker with PostgreSQL {}",
        version_tag
    );

    // Create a directory path for the host mount
    let backup_dir = Path::new(backup_path)
        .parent()
        .unwrap_or(Path::new("/tmp"))
        .to_string_lossy();
    let backup_file = Path::new(backup_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    // Build connection URL for pg_restore
    let connection_url = if let Some(pass) = password {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            username, pass, host, port, database
        )
    } else {
        format!("postgresql://{}@{}:{}/{}", username, host, port, database)
    };

    // Build Docker command
    let mut docker_cmd = Command::new("docker");
    docker_cmd
        .arg("run")
        .arg("--rm")
        .arg("--network=host") // Use host networking to connect to the PostgreSQL server
        .arg(format!("-v{}:/backup", backup_dir))
        .arg(format!("postgres:{}", version_tag))
        .arg("pg_restore")
        .arg("--dbname")
        .arg(connection_url)
        .arg("--verbose")
        .arg("--no-owner")
        .arg("--no-privileges")
        .arg("--clean") // Drop objects before recreating them
        .arg("--if-exists") // Add IF EXISTS for cleaner drops
        .arg("--no-comments") // Skip comments to avoid warnings
        .arg("--exit-on-error") // Continue on error
        .arg(format!("/backup/{}", backup_file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    debug!("Executing Docker pg_restore command");
    let output = docker_cmd
        .output()
        .context("Failed to execute Docker command")?;

    // Check for errors, but continue on warnings
    if !output.status.success() {
        // Log the error but don't fail completely if there's output on stdout indicating progress
        warn!(
            "Docker pg_restore returned non-zero exit code: {:?}",
            output.status.code()
        );
        warn!(
            "Docker pg_restore stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // If we see serious errors, fail the restore
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("ERROR:") && !stderr.contains("WARNING:") {
            return Err(anyhow::anyhow!("Docker pg_restore failed with errors"));
        }
    }

    info!("Database restore completed via Docker");
    Ok(())
}
