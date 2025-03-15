use anyhow::Result;
use clap::Parser;
use log::{info, warn, LevelFilter};

mod cli;
use cli::{Cli, Commands};
use vprs3bkp_core::databases::{
    backup_source,
    configs::{PGSourceConfig, SourceConfig}, restore_source,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    let _ = dotenv::dotenv();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Set up logging
    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();

    match cli.command {
        Commands::BackupPostgres {
            database,
            host,
            port,
            username,
            password,
            compression,
        } => {
            let source_config = SourceConfig::PG(PGSourceConfig {
                name: "".into(),
                database,
                host,
                port,
                username,
                password,
            });

            backup_source(source_config).await?;
        }
        Commands::RestorePostgres {
            database,
            host,
            port,
            username,
            password,
            key,
            latest,
            force_docker,
            drop_db,
        } => {
            let source_config = SourceConfig::PG(PGSourceConfig {
                name: "".into(),
                database,
                host,
                port,
                username,
                password,
            });

            restore_source(source_config, dump_data)

            info!("PostgreSQL database restore completed successfully");
        }
        Commands::List {
            backup_type,
            database,
            latest_only,
            limit,
        } => {
            info!("Listing backups in bucket: {}/{}", cli.bucket, cli.prefix);

            let backups = list_backups(
                &s3_client,
                &cli.bucket,
                &cli.prefix,
                backup_type.as_deref(),
                database.as_deref(),
                limit,
            )
            .await?;

            if backups.is_empty() {
                info!("No backups found matching the criteria");
                return Ok(());
            }

            let backups_to_display = if latest_only {
                info!("Showing only the latest backup per database");
                get_latest_backups_by_db(&backups)
            } else {
                backups.iter().collect()
            };

            // Find the maximum width needed for each column
            let mut max_key_width = 4; // "Key".len()
            let mut max_db_width = 8; // "Database".len()
            let mut max_type_width = 4; // "Type".len()
            let date_width = 16; // Fixed width for "YYYY-MM-DD HH:MM" format

            // Calculate the required width for each column based on actual data
            for backup in &backups_to_display {
                max_key_width = max_key_width.max(backup.key.len());
                max_db_width = max_db_width.max(backup.db_name.len());
                max_type_width = max_type_width.max(backup.backup_type.len());
            }

            // Cap maximum widths to keep table reasonable
            max_key_width = max_key_width.min(60); // Cap key length at 60 chars
            max_db_width = max_db_width.min(20); // Cap db name at 20 chars
            max_type_width = max_type_width.min(15); // Cap type at 15 chars

            // Calculate the total width of the table
            let total_width = max_key_width + max_db_width + max_type_width + date_width + 9; // 9 for separators and padding

            println!("\nAvailable backups:");
            println!("{:-<width$}", "", width = total_width);
            println!(
                "{:<key_width$} | {:<db_width$} | {:<type_width$} | {:<date_width$}",
                "Key",
                "Database",
                "Type",
                "Date",
                key_width = max_key_width,
                db_width = max_db_width,
                type_width = max_type_width,
                date_width = date_width
            );
            println!("{:-<width$}", "", width = total_width);

            for backup in &backups_to_display {
                // Handle potential truncation for the key (if we capped max width)
                let display_key = if backup.key.len() > max_key_width {
                    // Show beginning and end with ellipsis in the middle
                    let start = &backup.key[..max_key_width / 3];
                    let end = &backup.key[backup.key.len() - max_key_width / 3..];
                    format!("{}...{}", start, end)
                } else {
                    backup.key.clone()
                };

                // Format the timestamp in a friendly way
                let friendly_date = format_timestamp(&backup.timestamp);

                println!(
                    "{:<key_width$} | {:<db_width$} | {:<type_width$} | {:<date_width$}",
                    display_key,
                    backup.db_name,
                    backup.backup_type,
                    friendly_date,
                    key_width = max_key_width,
                    db_width = max_db_width,
                    type_width = max_type_width,
                    date_width = date_width
                );
            }

            println!("{:-<width$}", "", width = total_width);
            println!("Total: {} backups", backups_to_display.len());
        }
    }

    Ok(())
}
