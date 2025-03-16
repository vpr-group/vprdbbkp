use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfigName(pub String);

pub trait BaseStorageConfig: std::fmt::Debug {
    fn name(&self) -> &str;
    fn provider_type(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStorageConfig {
    pub name: String,
    pub root: PathBuf,
    pub prefix: Option<String>,
}

impl BaseStorageConfig for LocalStorageConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> &str {
        "local"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3StorageConfig {
    pub name: String,
    pub bucket: String,
    pub region: String,
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub prefix: Option<String>,
}

impl BaseStorageConfig for S3StorageConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> &str {
        "s3"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StorageConfig {
    #[serde(rename = "local")]
    Local(LocalStorageConfig),
    #[serde(rename = "s3")]
    S3(S3StorageConfig),
}

impl StorageConfig {
    pub fn name(&self) -> &str {
        match self {
            StorageConfig::Local(config) => config.name(),
            StorageConfig::S3(config) => config.name(),
        }
    }

    pub fn provider_type(&self) -> &str {
        match self {
            StorageConfig::Local(config) => config.provider_type(),
            StorageConfig::S3(config) => config.provider_type(),
        }
    }
}
