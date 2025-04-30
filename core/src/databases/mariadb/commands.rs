use super::{installer::MariaDBInstaller, version::MariaDBVersion};
use anyhow::{anyhow, Result};
use std::{env, fs, path::PathBuf};
use tokio::process::Command;

pub struct CommandBuilder {
    pub cache_dir: PathBuf,
    pub version: MariaDBVersion,
    pub database: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
}

impl CommandBuilder {
    pub fn new(
        version: MariaDBVersion,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("mariadb_tools_cache");

        fs::create_dir_all(&cache_dir)?;

        Ok(CommandBuilder {
            cache_dir,
            version,
            database: database.into(),
            host: host.into(),
            port,
            username: username.into(),
            password: password.map(|p| p.to_string()),
        })
    }

    pub async fn get_base_path(&self) -> Result<PathBuf> {
        let base_path = self.cache_dir.join(format!("{}", self.version.as_str()));
        return Ok(base_path);
    }

    pub async fn get_bin_path(&self) -> Result<PathBuf> {
        let path = self.get_base_path().await?.join("bin");
        Ok(path)
    }

    pub async fn build_base_cmd(&self, bin_name: &str) -> Result<Command> {
        let path = self.get_bin_path().await?.join(bin_name);

        if !path.exists() {
            let installer = MariaDBInstaller::new(self.version);
            match installer.install(self.cache_dir.clone()).await {
                Ok(path) => {
                    if path.exists() {
                        println!(
                            "Successfully installed MariaDB {} to {}",
                            self.version,
                            path.display()
                        )
                    } else {
                        return Err(anyhow!("Binary for MariaDB {} not found", self.version));
                    }
                }
                Err(error) => return Err(anyhow!("Failed to install MariaDB {}", error)),
            };
        }

        let mut cmd = Command::new(&path);

        // Common arguments
        cmd.arg("-h")
            .arg(&self.host)
            .arg("-P")
            .arg(self.port.to_string())
            .arg("-u")
            .arg(&self.username);

        if let Some(password) = &self.password {
            cmd.env("MYSQL_PWD", password.as_str());
        }

        Ok(cmd)
    }

    pub async fn build_connection_check_command(&self) -> Result<Command> {
        let mut cmd = self.build_base_cmd("mariadb").await?;

        cmd.arg(&self.database)
            .arg("--connect-timeout=5")
            .arg("-e")
            .arg("SELECT 1");

        Ok(cmd)
    }

    pub async fn build_version_check_command(&self) -> Result<Command> {
        let mut cmd = self.build_base_cmd("mariadb").await?;

        cmd.arg(&self.database)
            .arg("--batch")
            .arg("--skip-column-names")
            .arg("-e")
            .arg("SELECT VERSION();");

        Ok(cmd)
    }

    pub async fn build_dump_command(&self, compression: Option<u8>) -> Result<Command> {
        let mut cmd = self.build_base_cmd("mariadb-dump").await?;

        cmd.arg(format!("--host={}", self.host))
            .arg(format!("--port={}", self.port))
            .arg(format!("--user={}", self.username))
            .arg("--single-transaction")
            .arg("--routines")
            .arg("--triggers")
            .arg("--events")
            .arg("--complete-insert")
            .arg(&self.database);

        if compression.is_none() {
            cmd.arg("--compress");
        }

        Ok(cmd)
    }

    pub async fn build_connection_command(&self) -> Result<Command> {
        let mut cmd = self.build_base_cmd("mariadb").await?;

        cmd.arg(&self.database)
            .arg("--batch") // Output results in tab-separated format
            .arg("--skip-column-names")
            .arg("-e");

        Ok(cmd)
    }
}
