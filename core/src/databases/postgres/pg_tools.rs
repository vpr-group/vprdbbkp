use anyhow::anyhow;
use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};
use tokio::process::Command;

use super::pg_versions::PostgresVersion;

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
        // Build the path to the psql binary
        let psql_path = self
            .cache_dir
            .join(format!("{}/bin/psql", self.version.as_str()));

        // Prepare the command
        let mut cmd = Command::new(&psql_path);

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

        // If password is provided, set it as an environment variable
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pg_tools_initialization() {
        let pg_tools = PgTools::new(PostgresVersion::V16).expect("Failed to initialize PgTools");
        assert!(pg_tools.cache_dir.exists());
        assert_eq!(pg_tools.version, PostgresVersion::V16);
    }

    #[tokio::test]
    async fn test_pg_tool_detect_version() {
        let pg_tools = PgTools::new(PostgresVersion::V12).expect("Failed to initialize PgTools");
        let version = pg_tools
            .get_postgres_version("api", "localhost", 5432, "postgres", Some("postgres"))
            .await
            .expect("Failed to get postgres version");

        assert_eq!(version, PostgresVersion::V17);
    }
}
