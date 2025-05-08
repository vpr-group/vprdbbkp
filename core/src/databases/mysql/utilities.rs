use std::{env, path::PathBuf};

use crate::databases::UtilitiesTrait;
use anyhow::{anyhow, Result};
use dirs::cache_dir;
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
}

#[async_trait]
impl UtilitiesTrait for MySqlUtilities {
    fn get_base_path(&self) -> Result<PathBuf> {
        let path = cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("vprdbbkp")
            .join("mysql")
            .join(self.version.to_string())
            .join("bin");

        Ok(path)
    }

    async fn get_command(&self, bin_name: &str) -> Result<Command> {
        let base_path = self.get_base_path()?;
        let bin_path = base_path.join(bin_name);

        if !bin_path.exists() {
            return Err(anyhow!("{} binary not found", bin_path.display()));
        }

        let command = Command::new(&bin_path);
        Ok(command)
    }
}
