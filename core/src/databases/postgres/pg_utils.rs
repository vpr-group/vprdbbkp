use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn get_postgres_base_directory() -> Result<PathBuf> {
    // First, check for PGDATA environment variable which points to data directory
    if let Ok(pgdata) = env::var("PGDATA") {
        let pgdata_path = PathBuf::from(pgdata);
        if pgdata_path.exists() {
            // PGDATA is the data directory, not the base directory
            // Try to find the base directory by going up one level from bin or lib
            if let Some(parent) = pgdata_path.parent() {
                return Ok(parent.to_path_buf());
            }
        }
    }

    // Check for PostgreSQL installed through common package managers by running pg_config
    let pg_config_output = Command::new("pg_config").arg("--bindir").output();

    if let Ok(output) = pg_config_output {
        if output.status.success() {
            let bin_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = PathBuf::from(bin_dir);
            if let Some(parent) = path.parent() {
                if let Some(parent) = parent.parent() {
                    return Ok(parent.to_path_buf());
                }

                return Ok(parent.to_path_buf());
            }
        }
    }

    // Platform-specific fallback paths
    if cfg!(target_os = "macos") {
        // Check common macOS PostgreSQL installation locations
        let macos_paths = vec![
            "/usr/local/pgsql",
            "/usr/local/postgresql",
            "/opt/homebrew/opt/postgresql",
            "/Applications/Postgres.app/Contents/Versions",
        ];

        for path in macos_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return Ok(path_buf);
            }
        }

        // Check for PostgreSQL installed through Homebrew
        let brew_output = Command::new("brew")
            .args(["--prefix", "postgresql"])
            .output();

        if let Ok(output) = brew_output {
            if output.status.success() {
                let brew_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(brew_path));
            }
        }
    } else if cfg!(target_os = "linux") {
        // Check common Linux PostgreSQL installation locations
        let linux_paths = vec![
            "/usr/lib/postgresql",
            "/usr/local/pgsql",
            "/var/lib/postgresql",
            "/opt/postgresql",
        ];

        for path in linux_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return Ok(path_buf);
            }
        }
    } else if cfg!(target_os = "windows") {
        // Check common Windows PostgreSQL installation locations
        let program_files =
            env::var("ProgramFiles").unwrap_or_else(|_| String::from("C:\\Program Files"));
        let program_files_x86 = env::var("ProgramFiles(x86)")
            .unwrap_or_else(|_| String::from("C:\\Program Files (x86)"));

        let windows_paths = vec![
            format!("{}\\PostgreSQL", program_files),
            format!("{}\\PostgreSQL", program_files_x86),
        ];

        for base_path in windows_paths {
            let base_path_buf = PathBuf::from(&base_path);
            if base_path_buf.exists() {
                // If we couldn't find a version subdirectory, return the base path
                return Ok(base_path_buf);
            }
        }
    }

    Err(anyhow!("Could not find PostgreSQL installation directory"))
}
