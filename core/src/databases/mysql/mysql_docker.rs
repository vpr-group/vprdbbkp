use anyhow::{Context, Result};
use log::{debug, error, info};
use std::process::{Command, Stdio};

/// Check if Docker is available
pub fn is_docker_available() -> bool {
    let result = Command::new("docker").arg("--version").output();
    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Get MySQL server version by querying the server
pub async fn get_mysql_version(
    _database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
) -> Result<(u32, u32)> {
    // First try with mysql client
    let version = get_mysql_version_with_client(host, port, username, password).await;

    // If mysql client works, use that version
    if let Ok(v) = version {
        return Ok(v);
    }

    // If mysql client fails, try to detect version from error message or just return a default
    debug!("Could not detect MySQL version with client, using default version");
    // Return a default version that's likely to have Docker images available
    Ok((8, 0))
}

/// Try to get MySQL version using mysql client
async fn get_mysql_version_with_client(
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
) -> Result<(u32, u32)> {
    let mut mysql_cmd = Command::new("mysql");
    mysql_cmd
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--user={}", username))
        .arg("--protocol=tcp") // Force TCP protocol
        .arg("--execute=SELECT VERSION();")
        .stdout(Stdio::piped());

    if let Some(password) = password {
        mysql_cmd.arg(format!("--password={}", password));
    }

    debug!("Executing mysql command to get server version");
    let output = mysql_cmd
        .output()
        .context("Failed to execute mysql command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to get MySQL version: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    parse_mysql_version(&version_output)
}

/// Parse MySQL version string
fn parse_mysql_version(version_output: &str) -> Result<(u32, u32)> {
    // Example: "VERSION()\n8.0.32-0ubuntu0.22.04.2\n"
    debug!("Raw MySQL version output: {}", version_output);

    // Try first with standard format
    let re = regex::Regex::new(r"(\d+)\.(\d+)(?:\.|\-)").context("Failed to compile regex")?;

    if let Some(caps) = re.captures(version_output) {
        let major: u32 = caps
            .get(1)
            .map(|m| m.as_str().parse())
            .context("Failed to parse major version")?
            .context("Failed to parse major version as u32")?;

        let minor: u32 = caps
            .get(2)
            .map(|m| m.as_str().parse())
            .context("Failed to parse minor version")?
            .context("Failed to parse minor version as u32")?;

        debug!("Parsed MySQL version: {}.{}", major, minor);
        return Ok((major, minor));
    }

    // Fallback to just extract any numbers we can find
    let fallback_re = regex::Regex::new(r"(\d+)").context("Failed to compile fallback regex")?;
    let numbers: Vec<u32> = fallback_re
        .captures_iter(version_output)
        .filter_map(|cap| cap.get(1).and_then(|m| m.as_str().parse::<u32>().ok()))
        .collect();

    if numbers.len() >= 2 {
        debug!(
            "Fallback parsed MySQL version: {}.{}",
            numbers[0], numbers[1]
        );
        return Ok((numbers[0], numbers[1]));
    } else if numbers.len() == 1 {
        debug!(
            "Fallback parsed MySQL version (only major): {}.0",
            numbers[0]
        );
        return Ok((numbers[0], 0));
    }

    Err(anyhow::anyhow!(
        "Failed to parse MySQL version: {}",
        version_output
    ))
}

/// Execute mysqldump using a Docker container with matching MySQL version
pub async fn docker_mysqldump(
    database: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    compression: u8,
    server_version: (u32, u32),
) -> Result<Vec<u8>> {
    let (major, _minor) = server_version;

    // Map version to available Docker image tags
    // MySQL only publishes major version tags for their Docker images
    let mysql_image = match major {
        5 => "mysql:5.7",    // Latest 5.x version
        8 => "mysql:8.0",    // Latest 8.x version
        9 => "mysql:8.0",    // If version 9, fall back to 8.0 for now
        10 => "mysql:8.0",   // If version 10, fall back to 8.0 for now
        11 => "mysql:8.0",   // If version 11, fall back to 8.0 for now
        _ => "mysql:latest", // Default to latest for unknown versions
    };

    info!(
        "Using Docker image: {} for backup (server version: {}.{})",
        mysql_image, server_version.0, server_version.1
    );

    info!("Using Docker image: {} for backup", mysql_image);

    // Build the docker run command
    let mut docker_cmd = Command::new("docker");
    docker_cmd
        .arg("run")
        .arg("--rm")
        .arg("--network=host") // Use host network to connect to the MySQL server
        .arg("-i") // Interactive to allow piping
        .arg(&mysql_image)
        .arg("mysqldump")
        .arg(format!("--host={}", host))
        .arg(format!("--port={}", port))
        .arg(format!("--user={}", username))
        .arg("--protocol=tcp") // Force TCP protocol
        .arg("--single-transaction")
        .arg("--routines")
        .arg("--triggers")
        .arg("--events")
        .arg("--set-gtid-purged=OFF")
        .arg("--no-tablespaces") // Avoid accessing TABLESPACES
        .arg("--column-statistics=0") // Disable column statistics
        .arg(database)
        .stdout(Stdio::piped());

    if let Some(password) = password {
        docker_cmd.arg(format!("--password={}", password));
    }

    // Run gzip in a separate process
    let mut gzip_cmd = Command::new("gzip");
    gzip_cmd
        .arg(format!("-{}", compression))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    // Spawn the Docker command
    debug!("Executing Docker mysqldump command");
    let mut docker_process = docker_cmd
        .spawn()
        .context("Failed to execute Docker command")?;

    // Spawn the gzip command
    debug!("Executing gzip command");
    let mut gzip_process = gzip_cmd.spawn().context("Failed to execute gzip")?;

    // Get stdout from Docker process
    let docker_stdout = docker_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture Docker stdout"))?;

    // Get stdin for gzip process
    let mut gzip_stdin = gzip_process
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture gzip stdin"))?;

    // Pipe Docker stdout to gzip stdin
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

    // Check process exit statuses
    let docker_status = docker_process
        .wait()
        .context("Failed to wait for Docker process")?;
    if !docker_status.success() {
        error!(
            "Docker mysqldump failed with exit code: {:?}",
            docker_status.code()
        );
        return Err(anyhow::anyhow!("Docker mysqldump command failed"));
    }

    let gzip_status = gzip_process.wait().context("Failed to wait for gzip")?;
    if !gzip_status.success() {
        error!("gzip failed with exit code: {:?}", gzip_status.code());
        return Err(anyhow::anyhow!("gzip command failed"));
    }

    info!(
        "Docker backup completed successfully, size: {} bytes",
        output.len()
    );
    Ok(output)
}
