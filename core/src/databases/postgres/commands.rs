use anyhow::{anyhow, Result};
use std::{env, fs, path::PathBuf};
use tokio::process::Command;

use crate::databases::DbVersion;

use super::{installer::PgInstaller, version::PostgreSQLVersion};

pub struct CommandBuilder {
    pub cache_dir: PathBuf,
    pub version: PostgreSQLVersion,
    pub database: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
}

impl CommandBuilder {
    pub fn new(
        version: PostgreSQLVersion,
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("postgresql_tools_cache");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
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
            let installer = PgInstaller::new(self.version);
            match installer.install(self.cache_dir.clone()).await {
                Ok(path) => {
                    if path.exists() {
                        println!(
                            "Successfully installed PostgreSQL {} to {}",
                            self.version,
                            path.display()
                        )
                    } else {
                        return Err(anyhow!("Binary for PostgreSQL {} not found", self.version));
                    }
                }
                Err(error) => return Err(anyhow!("Failed to install PostgreSQL {}", error)),
            };
        }

        let mut cmd = Command::new(&path);

        cmd.arg("-h")
            .arg(&self.host)
            .arg("-p")
            .arg(self.port.to_string())
            .arg("-U")
            .arg(&self.username)
            .arg("-d")
            .arg(&self.database);

        if let Some(pass) = &self.password {
            cmd.env("PGPASSWORD", pass);
        }

        Ok(cmd)
    }

    pub async fn build_connection_check_command(&self) -> Result<Command> {
        let mut cmd = self.build_base_cmd("psql").await?;

        cmd.arg("-c")
            .arg("SELECT 1")
            .arg("-v")
            .arg("TIMEOUT=5")
            .arg("--set")
            .arg("statement_timeout=5000");

        Ok(cmd)
    }

    pub async fn build_dump_command(&self) -> Result<Command> {
        let mut cmd = self.build_base_cmd("pg_dump").await?;

        cmd.arg("--format=custom")
            .arg("--schema=*")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--no-owner")
            .arg("--blobs")
            .arg("--exclude-schema=information_schema")
            .arg("--exclude-schema=pg_catalog")
            .arg("--exclude-schema=pg_toast")
            .arg("--exclude-schema=pg_temp*")
            .arg("--exclude-schema=pg_toast_temp*");

        Ok(cmd)
    }

    pub async fn build_connection_command(&self) -> Result<Command> {
        let mut cmd = self.build_base_cmd("psql").await?;

        cmd.arg("--no-psqlrc")
            .arg("-v")
            .arg("ON_ERROR_STOP=1")
            .arg("-t")
            .arg("-c");

        Ok(cmd)
    }

    pub async fn build_restore_command(&self) -> Result<Command> {
        let mut cmd = self.build_base_cmd("pg_restore").await?;

        cmd.arg("--no-owner")
            .arg("--no-privileges")
            .arg("--no-comments")
            .arg("--no-acl")
            .arg("--single-transaction")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--exclude-schema=information_schema")
            .arg("--exclude-schema=pg_catalog")
            .arg("--exclude-schema=pg_toast")
            .arg("--exclude-schema=pg_temp*")
            .arg("--exclude-schema=pg_toast_temp*");

        Ok(cmd)
    }
}
