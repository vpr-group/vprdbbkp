use super::{
    mysql::connection::MySqlConnection, postgres::connection::PostgreSqlConnection,
    ssh_tunnel::SshTunnelConfig, DatabaseConnectionTrait,
};
use anyhow::Result;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ConnectionType {
    PostgreSql,
    MySql,
    // MariaDB,
}

#[derive(Debug, Clone)]
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
