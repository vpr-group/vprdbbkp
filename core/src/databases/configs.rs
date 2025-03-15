use serde::{Deserialize, Serialize};

pub trait BaseBackupSourceConfig {
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGSourceConfig {
    pub name: String,
    pub database: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
}

impl BaseBackupSourceConfig for PGSourceConfig {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    #[serde(rename = "pg")]
    PG(PGSourceConfig),
}
