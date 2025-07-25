use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use vprs3bkp_core::{
    databases::{
        ssh_tunnel::{SshAuthMethod, SshTunnelConfig},
        ConnectionType, DatabaseConfig,
    },
    storage::provider::{LocalStorageConfig, S3StorageConfig, StorageConfig},
};

mod tests;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Interactive,
    Backup(BackupArgs),
    Restore(RestoreArgs),
    List(ListArgs),
    Cleanup(CleanupArgs),
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    List,
    Create { name: String },
    Delete { name: String },
    Use { name: String },
    Active,
}

#[derive(Args, Debug)]
pub struct BackupArgs {
    #[arg(short, long, help = "Use workspace for configuration")]
    pub workspace: Option<String>,

    #[command(flatten)]
    pub database_config: DatabaseArgs,

    #[command(flatten)]
    pub storage_config: StorageArgs,

    #[arg(short, long, help = "Retention period (e.g. '30d', '1w', '6m')")]
    pub retention: Option<String>,
}

#[derive(Args, Debug)]
pub struct RestoreArgs {
    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub drop_database: bool,

    #[arg(long)]
    pub latest: bool,

    #[arg(short, long, help = "Use workspace for configuration")]
    pub workspace: Option<String>,

    #[command(flatten)]
    pub database_config: DatabaseArgs,

    #[command(flatten)]
    pub storage_config: StorageArgs,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(short, long)]
    pub database: Option<String>,

    #[arg(long)]
    pub latest_only: bool,

    #[arg(long, default_value = "10")]
    pub limit: Option<usize>,

    #[arg(short, long, help = "Use workspace for configuration")]
    pub workspace: Option<String>,

    #[command(flatten)]
    pub storage: StorageArgs,
}

#[derive(Args, Debug)]
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

    #[arg(short, long, help = "Use workspace for configuration")]
    pub workspace: Option<String>,

    #[command(flatten)]
    pub storage: StorageArgs,
}

#[derive(Args, Clone, Debug)]
pub struct SshArgs {
    #[arg(long)]
    pub ssh_host: Option<String>,

    #[arg(long)]
    ssh_username: Option<String>,

    #[arg(long)]
    ssh_key_path: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct DatabaseArgs {
    #[arg(long, help = "Database type ('postgresql' or 'mysql')")]
    pub database_type: Option<String>,

    #[arg(long)]
    pub database: Option<String>,

    #[arg(long)]
    pub host: Option<String>,

    #[arg(long)]
    pub port: Option<u16>,

    #[arg(long)]
    pub username: Option<String>,

    #[arg(long, env = "PGPASSWORD")]
    pub password: Option<String>,

    #[command(flatten)]
    pub ssh: Option<SshArgs>,
}

#[derive(Args, Debug)]
pub struct StorageArgs {
    #[arg(long, default_value = "local")]
    pub storage_type: Option<String>,

    #[arg(long, default_value = "default")]
    pub storage_name: Option<String>,

    #[arg(long)]
    pub location: Option<String>,

    #[arg(long, env = "S3_BUCKET")]
    pub bucket: Option<String>,

    #[arg(long, env = "S3_REGION", default_value = "us-east-1")]
    pub region: Option<String>,

    #[arg(long, env = "S3_ENDPOINT")]
    pub endpoint: Option<String>,

    #[arg(long, env = "S3_ACCESS_KEY_ID", env = "S3_ACCESS_KEY")]
    pub access_key: Option<String>,

    #[arg(long, env = "S3_SECRET_ACCESS_KEY", env = "S3_SECRET_KEY")]
    pub secret_key: Option<String>,
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

pub fn storage_from_cli(args: &StorageArgs) -> Result<StorageConfig> {
    let default_storage_type = "local".to_string();
    let storage_type = args.storage_type.as_ref().unwrap_or(&default_storage_type);
    match storage_type.as_str() {
        "s3" => {
            // Validate required args for S3
            let bucket = args
                .bucket
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --bucket parameter"))?;
            let endpoint = args
                .endpoint
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --endpoint parameter"))?;
            let access_key = args
                .access_key
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --access-key parameter"))?;
            let secret_key = args
                .secret_key
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --secret-key parameter"))?;
            let region = args
                .region
                .clone()
                .ok_or_else(|| anyhow!("S3 storage requires --region parameter"))?;

            Ok(StorageConfig::S3(S3StorageConfig {
                name: args
                    .storage_name
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                bucket,
                region,
                endpoint: Some(endpoint),
                access_key,
                secret_key,
                location: args
                    .location
                    .clone()
                    .ok_or_else(|| anyhow!("Location is required"))?,
                id: "".into(),
            }))
        }
        "local" => Ok(StorageConfig::Local(LocalStorageConfig {
            name: args
                .storage_name
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            id: "".into(),
            location: args
                .location
                .clone()
                .ok_or_else(|| anyhow!("Location is required"))?,
        })),
        _ => Err(anyhow!("Unsupported storage type: {}", storage_type)),
    }
}

pub fn database_config_from_cli(args: &DatabaseArgs) -> Result<DatabaseConfig> {
    let database_type = args
        .database_type
        .as_ref()
        .ok_or_else(|| anyhow!("Database type is required"))?;
    let database = args
        .database
        .as_ref()
        .ok_or_else(|| anyhow!("Database name is required"))?;
    let host = args
        .host
        .as_ref()
        .ok_or_else(|| anyhow!("Host is required"))?;
    let port = args.port.ok_or_else(|| anyhow!("Port is required"))?;
    let username = args
        .username
        .as_ref()
        .ok_or_else(|| anyhow!("Username is required"))?;
    let ssh_tunnel = if let Some(ssh) = &args.ssh {
        let ssh_host = ssh
            .ssh_host
            .as_ref()
            .ok_or_else(|| anyhow!("SSH key path is required when using SSH tunnel"))?
            .clone();

        let ssh_key_path = ssh
            .ssh_key_path
            .as_ref()
            .ok_or_else(|| anyhow!("SSH key path is required when using SSH tunnel"))?
            .clone();

        let ssh_username = ssh
            .ssh_username
            .as_ref()
            .ok_or_else(|| anyhow!("SSH username is required when using SSH tunnel"))?
            .clone();

        Some(SshTunnelConfig {
            port: 22,
            host: ssh_host,
            username: ssh_username,
            auth_method: SshAuthMethod::PrivateKey {
                key_path: ssh_key_path,
                passphrase_key: None,
            },
        })
    } else {
        None
    };

    match database_type.as_str() {
        "postgresql" => Ok(DatabaseConfig {
            connection_type: ConnectionType::PostgreSql,
            database: database.clone(),
            id: "".into(),
            name: database.clone(),
            host: host.clone(),
            port,
            username: username.clone(),
            password: args.password.clone(),
            ssh_tunnel,
        }),
        "mysql" => Ok(DatabaseConfig {
            connection_type: ConnectionType::MySql,
            database: database.clone(),
            id: "".into(),
            name: database.clone(),
            host: host.clone(),
            port,
            username: username.clone(),
            password: args.password.clone(),
            ssh_tunnel,
        }),
        _ => Err(anyhow!("Unsupported database type: {}", database_type)),
    }
}
