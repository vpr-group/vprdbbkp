use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tools::PostgreSQLTools;

use super::DbAdapter;

pub mod commands;
pub mod installer;
mod tests;
pub mod tools;
pub mod version;

pub struct PostgreSQL {
    database: String,
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
}

impl PostgreSQL {
    pub fn new(
        database: &str,
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
    ) -> Self {
        PostgreSQL {
            database: database.into(),
            host: host.into(),
            port,
            username: username.into(),
            password: password.map(|p| p.to_string()),
        }
    }

    async fn get_tools(&self) -> Result<PostgreSQLTools> {
        let mut tools = PostgreSQLTools::default()?;
        let password_ref = self.password.as_deref();
        let version = tools
            .get_version(
                self.database.as_str(),
                self.host.as_str(),
                self.port,
                self.username.as_str(),
                password_ref,
            )
            .await?;

        tools = PostgreSQLTools::new(version)?;

        Ok(tools)
    }
}

#[async_trait]
impl DbAdapter for PostgreSQL {
    async fn is_connected(&self) -> Result<bool> {
        let tools = self.get_tools().await?;
        let password_ref = self.password.as_deref();
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
        let tools = self.get_tools().await?;
        let password_ref = self.password.as_deref();

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
        let tools = self.get_tools().await?;
        let password_ref = self.password.as_deref();

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
