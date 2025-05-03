use super::ssh_tunnel::SshTunnelConfig;

pub enum ConnectionType {
    PostgreSQL,
    MySQL,
    MariaDB,
}

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
