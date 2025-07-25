use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use mysql::connection::MySqlConnection;
use postgres::connection::PostgreSqlConnection;
use serde::{Deserialize, Serialize};
use ssh_tunnel::SshTunnelConfig;
use tokio::process::Command;
use version::Version;

pub mod mysql;
pub mod postgres;
pub mod ssh_tunnel;
pub mod version;

pub struct BackupOptions {
    // compression: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    pub drop_database_first: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    version: Version,
}

#[async_trait]
pub trait DatabaseConnectionTrait: Send + Sync + Unpin {
    async fn test(&self) -> Result<bool>;
    async fn get_metadata(&self) -> Result<DatabaseMetadata>;
    async fn backup(&self, writer: &mut (dyn Write + Send + Unpin)) -> Result<()>;
    async fn restore(&self, reader: &mut (dyn Read + Send + Unpin)) -> Result<()>;
    async fn restore_with_options(
        &self,
        reader: &mut (dyn Read + Send + Unpin),
        options: RestoreOptions,
    ) -> Result<()>;
}

#[async_trait]
pub trait UtilitiesTrait: Send + Sync + Unpin {
    fn get_base_path(&self) -> Result<PathBuf>;
    async fn get_command(&self, bin_name: &str) -> Result<Command>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    PostgreSql,
    MySql,
    // MariaDB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub id: String,
    pub name: String,
    pub connection_type: ConnectionType,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub ssh_tunnel: Option<SshTunnelConfig>,
}

pub struct DatabaseConnection {
    pub config: DatabaseConfig,
    pub connection: Arc<dyn DatabaseConnectionTrait>,
}

impl DatabaseConnection {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let connection: Arc<dyn DatabaseConnectionTrait> = match config.connection_type {
            ConnectionType::PostgreSql => {
                Arc::new(PostgreSqlConnection::new(config.clone()).await?)
            }
            ConnectionType::MySql => Arc::new(MySqlConnection::new(config.clone()).await?),
        };

        Ok(Self { config, connection })
    }
}
