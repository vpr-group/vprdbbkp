use std::borrow::Borrow;

use anyhow::Result;
use bytes::Bytes;
use configs::SourceConfig;
use postgres::{backup_postgres, is_postgres_connected, restore_postgres};

pub mod configs;
pub mod mysql;
pub mod postgres;

pub async fn backup_source<B>(source_config: B) -> Result<Bytes>
where
    B: Borrow<SourceConfig>,
{
    match source_config.borrow() {
        SourceConfig::PG(config) => {
            let bytes = backup_postgres(
                &config.database,
                &config.host,
                config.port,
                &config.username,
                Some(config.password.as_deref().unwrap_or("")),
                Some(8),
            )
            .await?;
            Ok(bytes)
        }
    }
}

pub async fn restore_source<B>(source_config: B, dump_data: Bytes) -> Result<()>
where
    B: Borrow<SourceConfig>,
{
    match source_config.borrow() {
        SourceConfig::PG(config) => {
            restore_postgres(
                &config.database,
                &config.host,
                config.port,
                &config.username,
                Some(config.password.as_deref().unwrap_or("")),
                dump_data,
                true,
            )
            .await?;
            Ok(())
        }
    }
}

pub async fn is_database_connected<B>(source_config: B) -> Result<bool>
where
    B: Borrow<SourceConfig>,
{
    match source_config.borrow() {
        SourceConfig::PG(config) => {
            let is_connected = is_postgres_connected(
                &config.database,
                &config.host,
                config.port,
                &config.username,
                Some(config.password.as_deref().unwrap_or("")),
            )
            .await?;

            Ok(is_connected)
        }
    }
}
