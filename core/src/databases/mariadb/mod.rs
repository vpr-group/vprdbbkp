use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tools::MariaDBTools;

use super::DbAdapter;

pub mod commands;
pub mod tests;
pub mod tools;
pub mod version;

pub struct MariaDB {
    database: String,
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl MariaDB {
    pub fn new(database: &str, host: &str, port: u16, username: &str, password: &str) -> Self {
        MariaDB {
            database: database.into(),
            host: host.into(),
            port,
            username: username.into(),
            password: password.into(),
        }
    }

    async fn get_tools(&self) -> Result<MariaDBTools> {
        let tools = MariaDBTools::with_detected_version(
            self.database.as_str(),
            self.host.as_str(),
            self.port,
            self.username.as_str(),
            self.password.as_str(),
        )
        .await?;

        Ok(tools)
    }
}

#[async_trait]
impl DbAdapter for MariaDB {
    async fn is_connected(&self) -> Result<bool> {
        let tools = self.get_tools().await?;
        let is_connected = tools
            .is_connected(
                self.database.as_str(),
                self.host.as_str(),
                self.port,
                self.username.as_str(),
                self.password.as_str(),
            )
            .await?;

        Ok(is_connected)
    }

    async fn dump(&self, compression: Option<u8>) -> Result<Bytes> {
        let tools = self.get_tools().await?;
        let output = tools
            .dump(
                self.database.as_str(),
                self.host.as_str(),
                self.port,
                self.username.as_str(),
                self.password.as_str(),
                compression,
            )
            .await?;

        Ok(output)
    }

    async fn restore(&self, dump_data: Bytes, compressed: bool, drop_database: bool) -> Result<()> {
        let tools = self.get_tools().await?;
        Ok(())
    }
}
