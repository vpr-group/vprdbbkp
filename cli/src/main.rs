use anyhow::Result;
use clap::Parser;
use cli::{database_config_from_cli, parse_retention, storage_from_cli, Cli, Commands};
use vprs3bkp_core::{
    databases::DatabaseConnection,
    storage::provider::{ListOptions, StorageProvider},
    DbBkp, RestoreOptions,
};

mod cli;
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup(args) => {
            let database_config = database_config_from_cli(&args.database_config)?;
            let storage_config = storage_from_cli(&args.storage_config)?;

            let database_connection = DatabaseConnection::new(database_config).await?;
            let storage_provider = StorageProvider::new(storage_config)?;

            let core = DbBkp::new(database_connection, storage_provider);

            // Test database & storage connection
            core.test().await?;

            let backup_file = core.backup().await?;

            println!("Backup completed successfully: {}", backup_file);
        }
        Commands::List(args) => {
            let storage_config = storage_from_cli(&args.storage)?;
            let storage_provider = StorageProvider::new(storage_config)?;
            storage_provider.test().await?;

            let entries = storage_provider
                .list_with_options(ListOptions {
                    latest_only: Some(args.latest_only),
                    limit: args.limit,
                })
                .await?;

            println!("Available backups:");

            for entry in entries {
                let filename = entry.name();
                let size = entry.metadata().content_length();
                let size_str = if size < 1024 {
                    format!("{}B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.2}KB", size as f64 / 1024.0)
                } else if size < 1024 * 1024 * 1024 {
                    format!("{:.2}MB", size as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{:.2}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                };

                println!("  {} ({})", filename, size_str);
            }
        }
        Commands::Restore(args) => {
            let database_config = database_config_from_cli(&args.database_config)?;
            let storage_config = storage_from_cli(&args.storage_config)?;

            let database_connection = DatabaseConnection::new(database_config).await?;
            let storage_provider = StorageProvider::new(storage_config)?;

            let core = DbBkp::new(database_connection, storage_provider);

            // Test database & storage connection
            core.test().await?;

            core.restore(RestoreOptions {
                name: args.name.clone(),
                compression_format: None,
                drop_database_first: Some(args.drop_database),
            })
            .await?;

            println!("Restore completed successfully: {}", args.name);
        }
        Commands::Cleanup(args) => {
            let storage_config = storage_from_cli(&args.storage)?;
            let storage = StorageProvider::new(storage_config)?;

            // Test storage connection
            storage.test().await?;

            let (entries_deleted, storage_reclaimed) = storage
                .cleanup(parse_retention(&args.retention)?, args.dry_run)
                .await?;

            println!(
                "{} Entries deleted, {} Storage reclaimed",
                entries_deleted, storage_reclaimed
            );
        }
    };

    Ok(())
}
