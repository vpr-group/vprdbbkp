use std::path::PathBuf;

use crate::{
    common::{download_and_install_binaries, get_binaries_base_path},
    databases::{version::Version, UtilitiesTrait},
};
use anyhow::{anyhow, Result};
use log::debug;
use tokio::process::Command;

use super::version::MySqlVersion;
use async_trait::async_trait;

pub struct MySqlUtilities {
    version: MySqlVersion,
}

impl MySqlUtilities {
    pub fn new(version: MySqlVersion) -> Self {
        MySqlUtilities { version }
    }

    pub async fn install(&self) -> Result<()> {
        let path = download_and_install_binaries(&Version::MySql(self.version.clone())).await?;

        debug!(
            "Successfully installed MySql utilities at {}",
            path.display()
        );

        Ok(())
    }
}

#[async_trait]
impl UtilitiesTrait for MySqlUtilities {
    fn get_base_path(&self) -> Result<PathBuf> {
        let path = get_binaries_base_path(&Version::MySql(self.version.clone())).join("bin");
        Ok(path)
    }

    async fn get_command(&self, bin_name: &str) -> Result<Command> {
        let base_path = self.get_base_path()?;
        let bin_path = base_path.join(bin_name);

        if !bin_path.exists() {
            debug!("MySql utilities not found, attempting to download and install");
            self.install().await?;

            if !bin_path.exists() {
                return Err(anyhow!("Binary {} not found after installation", bin_name));
            }
        }

        let command = Command::new(&bin_path);
        Ok(command)
    }
}
