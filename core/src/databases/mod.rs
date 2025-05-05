use std::{
    borrow::Borrow,
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes::Bytes;
use configs::SourceConfig;
use mariadb::MariaDB;
use postgres::PostgreSQL;
use serde::{Deserialize, Serialize};
use tokio::{process::Command, time::timeout};
use version::Version;

use crate::tunnel::Tunnel;

pub mod configs;
pub mod connection;
pub mod mariadb;
pub mod mysql;
pub mod postgres;
pub mod ssh_tunnel;
pub mod version;

pub struct BackupOptions {
    compression: Option<u16>,
}

pub struct RestoreOptions {}

// #[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    version: Version,
}

#[async_trait]
pub trait SQLDatabaseConnection: Send + Sync + Unpin {
    async fn test(&self) -> Result<bool>;
    async fn get_metadata(&self) -> Result<DatabaseMetadata>;
    async fn backup(&self, writer: &mut (dyn Write + Send)) -> Result<()>;
    async fn restore(&self, reader: &mut (dyn Read + Send)) -> Result<()>;
}

#[async_trait]
pub trait UtilitiesTrait: Send + Sync + Unpin {
    fn get_base_path(&self) -> Result<PathBuf>;
    async fn get_command(&self, bin_name: &str) -> Result<Command>;
}

#[async_trait]
pub trait DbAdapter: Send + Sync {
    async fn is_connected(&self) -> Result<bool>;
    async fn dump(&self) -> Result<Bytes>;
    async fn restore(&self, dump_data: Bytes, drop_database: bool) -> Result<()>;
}

pub trait DbVersion: Send + Sync + Sized {
    fn as_str(&self) -> &'static str;
    fn from_str(version: &str) -> Option<Self>;
    fn from_version_tuple(major: u32, minor: u32, _patch: u32) -> Option<Self>;
    fn parse_string_version(version_string: &str) -> Option<Self>;
}

pub fn get_db_adapter<B>(source_config: B) -> Box<dyn DbAdapter>
where
    B: Borrow<SourceConfig>,
{
    match source_config.borrow() {
        SourceConfig::PG(config) => Box::new(PostgreSQL::new(
            &config.database,
            &config.host,
            config.port,
            &config.username,
            Some(config.password.as_deref().unwrap_or("")),
        )),
        SourceConfig::MariaDB(config) => Box::new(MariaDB::new(
            &config.database,
            &config.host,
            config.port,
            &config.username,
            Some(config.password.as_deref().unwrap_or("")),
        )),
    }
}

pub async fn get_source_config_with_tunnel<B>(
    source_config: B,
) -> Result<(SourceConfig, Option<Tunnel>)>
where
    B: Borrow<SourceConfig>,
{
    let borrowed_config = source_config.borrow();
    let cloned_config = borrowed_config.clone();

    let tunnel_config = match borrowed_config {
        SourceConfig::PG(config) => config.tunnel_config.clone(),
        SourceConfig::MariaDB(config) => config.tunnel_config.clone(),
    };

    if let Some(tunnel_config) = tunnel_config {
        if tunnel_config.use_tunnel {
            let mut tunnel = Tunnel::new(tunnel_config.clone());
            tunnel.establish_tunnel(&cloned_config).await?;
            let new_source_config = tunnel.get_tunneled_config(&cloned_config);

            return match new_source_config {
                Some(source_config) => Ok((source_config, Some(tunnel))),
                None => Err(anyhow!("Unable to get a tunneled config for this source")),
            };
        }
    }

    Ok((cloned_config, None))
}

pub async fn backup<B>(source_config: B) -> Result<Bytes>
where
    B: Borrow<SourceConfig>,
{
    let (source_config, tunnel) = get_source_config_with_tunnel(source_config).await?;

    let db_adapter = get_db_adapter(&source_config);
    let bytes = db_adapter.dump().await?;

    if let Some(mut tunnel) = tunnel {
        tunnel.close_tunnel().await?;
    }

    Ok(bytes)
}

pub async fn restore<B>(source_config: B, dump_data: Bytes, drop_database: bool) -> Result<()>
where
    B: Borrow<SourceConfig>,
{
    let (source_config, tunnel) = get_source_config_with_tunnel(source_config).await?;

    let db_adapter = get_db_adapter(&source_config);
    db_adapter.restore(dump_data, drop_database).await?;

    if let Some(mut tunnel) = tunnel {
        tunnel.close_tunnel().await?;
    }

    Ok(())
}

pub async fn is_connected<B>(source_config: B) -> Result<bool>
where
    B: Borrow<SourceConfig>,
{
    match timeout(Duration::from_secs(5), async {
        let (source_config, tunnel) = get_source_config_with_tunnel(source_config).await?;

        let db_adapter = get_db_adapter(&source_config);
        let is_connected = db_adapter.is_connected().await?;

        if let Some(mut tunnel) = tunnel {
            tunnel.close_tunnel().await?;
        }

        Ok(is_connected)
    })
    .await
    {
        Ok(result) => result,
        Err(_) => Ok(false),
    }
}
