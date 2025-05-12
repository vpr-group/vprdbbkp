use std::path::PathBuf;

use crate::{
    common::{download_and_install_binaries, get_binaries_base_path},
    databases::{version::Version, UtilitiesTrait},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::debug;
use tokio::process::Command;

use super::version::PostgreSQLVersion;

pub struct PostgreSqlUtilities {
    version: PostgreSQLVersion,
}

impl PostgreSqlUtilities {
    pub fn new(version: PostgreSQLVersion) -> Self {
        PostgreSqlUtilities { version }
    }

    async fn install(&self) -> Result<()> {
        let path =
            download_and_install_binaries(&Version::PostgreSQL(self.version.clone())).await?;

        debug!(
            "Successfully installed PostgreSQL utilities at {}",
            path.display()
        );

        Ok(())
    }
}

#[async_trait]
impl UtilitiesTrait for PostgreSqlUtilities {
    fn get_base_path(&self) -> Result<PathBuf> {
        let path = get_binaries_base_path(&Version::PostgreSQL(self.version.clone())).join("bin");
        Ok(path)
    }

    async fn get_command(&self, bin_name: &str) -> Result<Command> {
        let base_path = self.get_base_path()?;
        let bin_path = base_path.join(bin_name);

        if !bin_path.exists() {
            debug!("PostgreSQL utilities not found, attempting to download and install");
            self.install().await?;

            if !bin_path.exists() {
                return Err(anyhow!("Binary {} not found after installation", bin_name));
            }
        }

        let command = Command::new(&bin_path);
        Ok(command)
    }
}
