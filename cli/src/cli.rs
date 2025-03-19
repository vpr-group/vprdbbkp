use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use vprs3bkp_core::{
    databases::configs::{PGSourceConfig, SourceConfig},
    storage::configs::{LocalStorageConfig, S3StorageConfig, StorageConfig},
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
}

#[derive(Args)]
pub struct BackupArgs {
    #[arg(short, long)]
    pub compression: Option<u8>,

    #[command(flatten)]
    pub source: SourceArgs,

    #[command(flatten)]
    pub storage: StorageArgs,
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

// Helper function to convert CLI arguments to storage config
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

// Helper function to convert CLI arguments to source config
pub fn source_from_cli(source: &SourceArgs) -> Result<SourceConfig> {
    match source.source_type.as_str() {
        "postgres" => Ok(SourceConfig::PG(PGSourceConfig {
            name: source.source_name.clone(),
            database: source.database.clone(),
            host: source.host.clone(),
            port: source.port,
            username: source.username.clone(),
            password: source.password.clone(),
        })),
        _ => Err(anyhow!("Unsupported source type: {}", source.source_type)),
    }
}
