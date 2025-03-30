use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use vprs3bkp_core::{
    databases::configs::{PGSourceConfig, SourceConfig},
    storage::configs::{LocalStorageConfig, S3StorageConfig, StorageConfig},
    tunnel::config::TunnelConfig,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Backup(BackupArgs),
    Restore(RestoreArgs),
    List(ListArgs),
    Cleanup(CleanupArgs),
}

#[derive(Args)]
pub struct BackupArgs {
    #[arg(short, long)]
    pub compression: Option<u8>,

    #[command(flatten)]
    pub source: SourceArgs,

    #[command(flatten)]
    pub storage: StorageArgs,

    #[arg(short, long, help = "Retention period (e.g. '30d', '1w', '6m')")]
    pub retention: Option<String>,
}

#[derive(Args)]
pub struct RestoreArgs {
    #[arg(short, long)]
    pub filename: Option<String>,

    #[arg(short, long)]
    pub drop_database: Option<bool>,

    #[arg(long)]
    pub latest: bool,

    #[command(flatten)]
    pub source: SourceArgs,

    #[command(flatten)]
    pub storage: StorageArgs,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(short, long)]
    pub database: Option<String>,

    #[arg(long)]
    pub latest_only: bool,

    #[arg(short, long, default_value = "10")]
    pub limit: usize,

    #[command(flatten)]
    pub storage: StorageArgs,
}

#[derive(Args)]
pub struct CleanupArgs {
    #[arg(short, long, help = "Retention period (e.g. '30d', '1w', '6m')")]
    pub retention: String,

    #[arg(
        long,
        help = "Only show which backups would be deleted without actually removing them"
    )]
    pub dry_run: bool,

    #[arg(short, long, help = "Database name to cleanup backups for")]
    pub database: Option<String>,

    #[command(flatten)]
    pub storage: StorageArgs,
}

#[derive(Args)]
pub struct SourceArgs {
    #[arg(long, default_value = "postgres")]
    pub source_type: String,

    #[arg(long, default_value = "default")]
    pub source_name: String,

    #[arg(short, long)]
    pub database: String,

    #[arg(long, short = 'H', default_value = "localhost")]
    pub host: String,

    #[arg(short, long, default_value = "5432")]
    pub port: u16,

    #[arg(short, long, default_value = "postgres")]
    pub username: String,

    #[arg(long, env = "PGPASSWORD")]
    pub password: Option<String>,

    #[arg(long)]
    pub use_ssh_tunnel: bool,

    #[arg(long)]
    pub ssh_key_path: Option<String>,

    #[arg(long)]
    pub ssh_username: Option<String>,
}

#[derive(Args)]
pub struct StorageArgs {
    // Shared args
    #[arg(long, default_value = "s3")]
    pub storage_type: String,

    #[arg(long, default_value = "default")]
    pub storage_name: String,

    #[arg(long)]
    pub prefix: Option<String>,

    // S3 specific args
    #[arg(long, env = "S3_BUCKET")]
    pub bucket: Option<String>,

    #[arg(long, env = "S3_REGION", default_value = "us-east-1")]
    pub region: String,

    #[arg(long, env = "S3_ENDPOINT")]
    pub endpoint: Option<String>,

    #[arg(long, env = "S3_ACCESS_KEY_ID", env = "S3_ACCESS_KEY")]
    pub access_key: Option<String>,

    #[arg(long, env = "S3_SECRET_ACCESS_KEY", env = "S3_SECRET_KEY")]
    pub secret_key: Option<String>,

    // Local specific args
    #[arg(long)]
    pub root: Option<PathBuf>,
}

pub fn parse_retention(retention: &str) -> Result<u64> {
    let len = retention.len();
    if len < 2 {
        return Err(anyhow!(
            "Invalid retention format. Use format like '30d', '4w', '2m', '1y'"
        ));
    }

    let value = retention[..len - 1]
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid retention value"))?;

    match retention.chars().last().unwrap() {
        'd' => Ok(value), // days
        'w' => Ok(value * 7), // weeks to days
        'm' => Ok(value * 30), // months to days (approximate)
        'y' => Ok(value * 365), // years to days (approximate)
        _ => Err(anyhow!("Invalid retention unit. Use 'd' for days, 'w' for weeks, 'm' for months, or 'y' for years")),
    }
}

pub fn storage_from_cli(storage: &StorageArgs) -> Result<StorageConfig> {
    match storage.storage_type.as_str() {
        "s3" => {
            // Validate required args for S3
            let bucket = storage
                .bucket
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --bucket parameter"))?;
            let endpoint = storage
                .endpoint
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --endpoint parameter"))?;
            let access_key = storage
                .access_key
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --access-key parameter"))?;
            let secret_key = storage
                .secret_key
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --secret-key parameter"))?;

            Ok(StorageConfig::S3(S3StorageConfig {
                name: storage.storage_name.clone(),
                bucket,
                region: storage.region.clone(),
                endpoint,
                access_key,
                secret_key,
                prefix: storage.prefix.clone(),
            }))
        }
        "local" => {
            // Validate required args for local
            let root = storage
                .root
                .clone()
                .ok_or_else(|| anyhow!("Local storage requires --root parameter"))?;

            Ok(StorageConfig::Local(LocalStorageConfig {
                name: storage.storage_name.clone(),
                root,
                prefix: storage.prefix.clone(),
            }))
        }
        _ => Err(anyhow!(
            "Unsupported storage type: {}",
            storage.storage_type
        )),
    }
}

pub fn source_from_cli(source: &SourceArgs) -> Result<SourceConfig> {
    let tunnel_config = if source.use_ssh_tunnel {
        let ssh_key_path = source
            .ssh_key_path
            .as_ref()
            .ok_or_else(|| anyhow!("SSH key path is required when using SSH tunnel"))?
            .clone();

        let ssh_username = source
            .ssh_username
            .as_ref()
            .ok_or_else(|| anyhow!("SSH username is required when using SSH tunnel"))?
            .clone();

        Some(TunnelConfig {
            use_tunnel: source.use_ssh_tunnel,
            key_path: ssh_key_path,
            username: ssh_username,
        })
    } else {
        None
    };

    match source.source_type.as_str() {
        "postgres" => Ok(SourceConfig::PG(PGSourceConfig {
            name: source.source_name.clone(),
            database: source.database.clone(),
            host: source.host.clone(),
            port: source.port,
            username: source.username.clone(),
            password: source.password.clone(),
            tunnel_config,
        })),
        _ => Err(anyhow!("Unsupported source type: {}", source.source_type)),
    }
}
