use std::{borrow::Borrow, time::Duration};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use configs::SourceConfig;
use postgres::{backup_postgres, is_postgres_connected, restore_postgres};
use tokio::time::timeout;

use crate::tunnel::Tunnel;

pub mod configs;
pub mod mariadb;
pub mod mysql;
pub mod postgres;

pub async fn get_source_config_with_tunnel<B>(
    source_config: B,
) -> Result<(SourceConfig, Option<Tunnel>)>
where
    B: Borrow<SourceConfig>,
{
    let borrowed_config = source_config.borrow();
    let cloned_config = borrowed_config.clone();

    match borrowed_config {
        SourceConfig::PG(config) => {
            if let Some(tunnel_config) = &config.tunnel_config {
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
        }
    }

    Ok((cloned_config, None))
}

pub async fn backup_source<B>(source_config: B) -> Result<Bytes>
where
    B: Borrow<SourceConfig>,
{
    let (source_config, tunnel) = get_source_config_with_tunnel(source_config).await?;

    match source_config {
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

            if let Some(mut tunnel) = tunnel {
                tunnel.close_tunnel().await?;
            }

            Ok(bytes)
        }
    }
}

pub async fn restore_source<B>(
    source_config: B,
    dump_data: Bytes,
    drop_database: bool,
) -> Result<()>
where
    B: Borrow<SourceConfig>,
{
    let (source_config, tunnel) = get_source_config_with_tunnel(source_config).await?;

    match source_config {
        SourceConfig::PG(config) => {
            restore_postgres(
                &config.database,
                &config.host,
                config.port,
                &config.username,
                Some(config.password.as_deref().unwrap_or("")),
                dump_data,
                true,
                drop_database,
            )
            .await?;

            if let Some(mut tunnel) = tunnel {
                tunnel.close_tunnel().await?;
            }

            Ok(())
        }
    }
}

pub async fn is_database_connected<B>(source_config: B) -> Result<bool>
where
    B: Borrow<SourceConfig>,
{
    match timeout(Duration::from_secs(5), async {
        let (source_config, tunnel) = get_source_config_with_tunnel(source_config).await?;
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
                if let Some(mut tunnel) = tunnel {
                    tunnel.close_tunnel().await?;
                }
                Ok(is_connected)
            }
        }
    })
    .await
    {
        Ok(result) => result,
        Err(_) => Ok(false),
    }
}
