use super::{
    postgres::connection::PostgreSQLConnection, ssh_tunnel::SshTunnelConfig, SQLDatabaseConnection,
};
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ConnectionType {
    PostgreSQL,
    // MySQL,
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
    pub connection: Arc<dyn SQLDatabaseConnection>,
}

impl DatabaseConnection {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let connection: Arc<dyn SQLDatabaseConnection> = match config.connection_type {
            ConnectionType::PostgreSQL => {
                Arc::new(PostgreSQLConnection::new(config.clone()).await?)
            }
        };

        Ok(Self { config, connection })
    }
}
