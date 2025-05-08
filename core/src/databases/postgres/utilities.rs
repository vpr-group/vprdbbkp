use std::{env, path::PathBuf};

use crate::databases::UtilitiesTrait;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dirs::cache_dir;
use tokio::process::Command;

use super::version::PostgreSQLVersion;

const BUCKET_URL: &str =
    "https://s3.pub1.infomaniak.cloud/object/v1/AUTH_f1ed7eb1a4594d268432025f27acb84f/postgres";

pub struct PostgreSqlUtilities {
    version: PostgreSQLVersion,
}

impl PostgreSqlUtilities {
    pub fn new(version: PostgreSQLVersion) -> Self {
        PostgreSqlUtilities { version }
    }

    async fn install(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl UtilitiesTrait for PostgreSqlUtilities {
    fn get_base_path(&self) -> Result<PathBuf> {
        let path = cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("vprdbbkp")
            .join("postgresql")
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
