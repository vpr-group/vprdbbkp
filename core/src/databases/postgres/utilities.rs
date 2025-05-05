use std::{env, path::PathBuf};

use crate::databases::UtilitiesTrait;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dirs::cache_dir;
use tokio::process::Command;

use super::version::PostgreSQLVersionV2;

const BUCKET_URL: &str =
    "https://s3.pub1.infomaniak.cloud/object/v1/AUTH_f1ed7eb1a4594d268432025f27acb84f/postgres";

pub struct Utilities {
    version: PostgreSQLVersionV2,
}

impl Utilities {
    pub fn new(version: PostgreSQLVersionV2) -> Self {
        Utilities { version }
    }
}

#[async_trait]
impl UtilitiesTrait for Utilities {
    fn get_base_path(&self) -> Result<PathBuf> {
        let path = cache_dir()
            .unwrap_or_else(|| env::temp_dir())
            .join("vprdbbkp")
            .join("postgresql");

        Ok(path)
    }

    fn get_command(&self, bin_name: &str) -> Result<Command> {
        let base_path = self.get_base_path()?;
        let bin_path = base_path.join(bin_name);

        if !bin_path.exists() {
            return Err(anyhow!("{} binary not found", bin_path.display()));
        }

        let command = Command::new(&bin_path);
        Ok(command)
    }

    async fn install(&self) -> Result<()> {
        Ok(())
    }
}
