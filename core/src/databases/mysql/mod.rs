use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tools::MySQLTools;

use super::DbAdapter;

pub mod commands;
pub mod installer;
pub mod tests;
pub mod tools;
pub mod version;

pub struct MySQL {
    database: String,
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
}

impl MySQL {
    pub fn new(
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Self {
        MySQL {
            database: database.into(),
            host: host.into(),
            port,
            username: username.into(),
            password: password.map(|p| p.to_string()),
        }
    }

    pub async fn get_tools(&self) -> Result<MySQLTools> {
        let password_ref = self.password.as_deref();
        let tools = MySQLTools::with_detected_version(
            self.database.as_str(),
            self.host.as_str(),
            self.port,
            self.username.as_str(),
            password_ref,
        )
        .await?;

        Ok(tools)
    }
}

#[async_trait]
impl DbAdapter for MySQL {
    async fn is_connected(&self) -> Result<bool> {
        let password_ref = self.password.as_deref();
        let tools = self.get_tools().await?;
        let is_connected = tools
            .is_connected(
                self.database.as_str(),
                self.host.as_str(),
                self.port,
                self.username.as_str(),
                password_ref,
            )
            .await?;

        Ok(is_connected)
    }

    async fn dump(&self) -> Result<Bytes> {
        let password_ref = self.password.as_deref();
        let tools = self.get_tools().await?;
        let output = tools
            .dump(
                self.database.as_str(),
                self.host.as_str(),
                self.port,
                self.username.as_str(),
                password_ref,
            )
            .await?;

        Ok(output)
    }

    async fn restore(&self, dump_data: Bytes, drop_database: bool) -> Result<()> {
        let password_ref = self.password.as_deref();
        let tools = self.get_tools().await?;
        tools
            .restore(
                self.database.as_str(),
                self.host.as_str(),
                self.port,
                self.username.as_str(),
                password_ref,
                dump_data,
                drop_database,
            )
            .await?;

        Ok(())
    }
}
