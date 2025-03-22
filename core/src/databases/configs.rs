use serde::{Deserialize, Serialize};

use crate::tunnel::config::TunnelConfig;

pub trait BaseSourceConfig {
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PGSourceConfig {
    pub name: String,
    pub database: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub tunnel_config: Option<TunnelConfig>,
}

impl BaseSourceConfig for PGSourceConfig {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SourceConfig {
    #[serde(rename = "pg")]
    PG(PGSourceConfig),
}
