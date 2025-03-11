use anyhow::{Context, Result};
use log::{debug, error, info};
use regex::Regex;
use std::process::{Command, Stdio};

/// Get the major and minor version of a PostgreSQL server
pub async fn get_postgres_version(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
) -> Result<(u32, u32)> {
    info!("Detecting PostgreSQL server version...");

    let mut psql_cmd = Command::new("psql");
    psql_cmd
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--username={}", username))
        .arg("--tuples-only")
        .arg("--no-align")
        .arg(format!(
            "--command=SELECT current_setting('server_version')"
        ))
        .arg(database)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(password) = password {
        psql_cmd.env("PGPASSWORD", password);
    }

    debug!("Executing psql to query server version");
    let output = psql_cmd
        .output()
        .context("Failed to execute psql command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("psql command failed: {}", stderr);
        return Err(anyhow::anyhow!(
            "Failed to query PostgreSQL version: {}",
            stderr
        ));
    }

    let version_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    debug!("Raw version output: {}", version_output);

    parse_postgres_version(&version_output)
}

/// Parse PostgreSQL version string into major and minor version numbers
fn parse_postgres_version(version_str: &str) -> Result<(u32, u32)> {
    // Match version formats like:
    // "9.6.24"
    // "10.23"
    // "11.22"
    // "12.17"
    // "13.14"
    // "14.10"
    // "15.5"
    // "16.2"
    // "17.4"
    let re = Regex::new(r"^(\d+)(?:\.(\d+))?").context("Failed to compile regex")?;

    if let Some(caps) = re.captures(version_str) {
        let major: u32 = caps
            .get(1)
            .map(|m| m.as_str().parse())
            .context("Failed to parse major version")?
            .context("Failed to parse major version as u32")?;

        let minor: u32 = caps
            .get(2)
            .map(|m| m.as_str().parse().unwrap_or(0))
            .unwrap_or(0);

        debug!("Parsed PostgreSQL version: {}.{}", major, minor);
        return Ok((major, minor));
    }

    Err(anyhow::anyhow!(
        "Failed to parse PostgreSQL version: {}",
        version_str
    ))
}

/// Run pg_dump through Docker with matching PostgreSQL version
pub async fn docker_pg_dump(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: u8,
    pg_version: (u32, u32),
) -> Result<Vec<u8>> {
    let (major, minor) = pg_version;
    let version_tag = if minor == 0 {
        format!("{}", major)
    } else {
        format!("{}.{}", major, minor)
    };

    info!("Using Docker with PostgreSQL {} for backup", version_tag);

    // Create connection URL with all parameters including password if provided
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
        .arg(format!("postgres:{}", version_tag))
        .arg("pg_dump")
        .arg("--format=custom")
        .arg(connection_url)
        .stdout(Stdio::piped());

    // Run pg_dump via Docker
    debug!("Executing pg_dump via Docker");
    let mut docker_process = docker_cmd
        .spawn()
        .context("Failed to execute Docker command")?;

    // Set up compression
    let mut gzip_cmd = Command::new("gzip");
    gzip_cmd
        .arg(format!("-{}", compression))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    debug!("Executing gzip command");
    let mut gzip_process = gzip_cmd.spawn().context("Failed to execute gzip")?;

    // Pipe Docker output to gzip
    let docker_stdout = docker_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture Docker stdout"))?;
    let mut gzip_stdin = gzip_process
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdin"))?;

    // Copy data between processes
    tokio::task::spawn_blocking(move || {
        std::io::copy(&mut std::io::BufReader::new(docker_stdout), &mut gzip_stdin)
            .context("Failed to pipe Docker output to gzip")
    });

    // Read compressed output
    let mut output = Vec::new();
    let mut gzip_stdout = gzip_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdout"))?;
    std::io::Read::read_to_end(&mut gzip_stdout, &mut output)?;

    // Check process exit status
    let docker_status = docker_process
        .wait()
        .context("Failed to wait for Docker process")?;
    if !docker_status.success() {
        error!(
            "Docker pg_dump failed with exit code: {:?}",
            docker_status.code()
        );
        return Err(anyhow::anyhow!("Docker pg_dump command failed"));
    }

    let gzip_status = gzip_process.wait().context("Failed to wait for gzip")?;
    if !gzip_status.success() {
        error!("gzip failed with exit code: {:?}", gzip_status.code());
        return Err(anyhow::anyhow!("gzip command failed"));
    }

    info!(
        "Backup via Docker completed successfully, size: {} bytes",
        output.len()
    );
    Ok(output)
}

/// Check if Docker is available on the system
pub fn is_docker_available() -> bool {
    let output = Command::new("docker")
        .arg("--version")
        .stdout(Stdio::null())
        .status();

    match output {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}
