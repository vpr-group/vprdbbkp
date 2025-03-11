use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use clap::Parser;
use folders::backup_folder;
use log::{info, warn, LevelFilter};

mod cli;
mod config;
mod databases;
mod folders;
mod s3;
mod utils;

use cli::{Cli, Commands};
use databases::{
    mysql,
    postgres::{self, pg_restore::restore_postgres},
};
use s3::{
    download_backup, get_latest_backup, get_latest_backups_by_db, list_backups, upload_to_s3,
};
use utils::{format_timestamp, get_backup_key};

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

    // Set up AWS configuration
    let region_provider =
        RegionProviderChain::first_try(cli.region.map(aws_sdk_s3::config::Region::new))
            .or_default_provider()
            .or_else("us-east-1");

    // Disable SSL verification if requested
    if cli.no_verify_ssl {
        warn!("SSL verification disabled for S3 connections - this is not recommended for production use");
        // Set environment variable to disable SSL verification
        std::env::set_var("AWS_HTTPS_VERIFY", "0");
    }

    // Create the AWS config
    let aws_config = aws_config::from_env().region(region_provider).load().await;

    // Build S3 client configuration
    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);

    // Add custom endpoint if specified
    if let Some(endpoint) = &cli.endpoint {
        info!("Using custom S3 endpoint: {}", endpoint);
        s3_config_builder = s3_config_builder.endpoint_url(endpoint);

        // Force path style access for custom endpoints
        info!("Enabling path-style access for S3-compatible service");
        s3_config_builder = s3_config_builder.force_path_style(true);
    }

    // Add explicit credentials if provided
    if let (Some(access_key), Some(secret_key)) = (&cli.access_key, &cli.secret_key) {
        info!("Using explicitly provided S3 credentials");

        // Create static credentials
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key,
            secret_key,
            None, // session token
            None, // expiry
            "explicit-credentials",
        );

        s3_config_builder = s3_config_builder.credentials_provider(credentials);
    }

    // Build the final S3 client with our configuration
    let s3_client = S3Client::from_conf(s3_config_builder.build());

    match cli.command {
        Commands::Postgres {
            database,
            host,
            port,
            username,
            password,
            compression,
            force_docker,
        } => {
            let backup_bytes = postgres::backup_postgres_with_options(
                &database,
                &host,
                port,
                &username,
                password.as_deref(),
                compression,
                force_docker,
            )
            .await?;

            let key = get_backup_key(&cli.prefix, "postgres", &database);
            upload_to_s3(
                &s3_client,
                &cli.bucket,
                &key,
                aws_sdk_s3::primitives::ByteStream::from(backup_bytes),
            )
            .await?;
        }
        Commands::Mysql {
            database,
            host,
            port,
            username,
            password,
            compression,
        } => {
            let backup_bytes = mysql::backup_mysql(
                &database,
                &host,
                port,
                &username,
                password.as_deref(),
                compression,
            )
            .await?;

            let key = get_backup_key(&cli.prefix, "mysql", &database);
            upload_to_s3(
                &s3_client,
                &cli.bucket,
                &key,
                aws_sdk_s3::primitives::ByteStream::from(backup_bytes),
            )
            .await?;
        }
        Commands::Folder {
            path,
            compress,
            compression_level,
            concurrency,
            include,
            exclude,
            skip_larger_than,
            add_timestamp,
        } => {
            info!("Starting folder backup for: {}", path);

            let stats = backup_folder(
                &s3_client,
                &cli.bucket,
                &cli.prefix,
                &path,
                compress,
                compression_level,
                concurrency,
                include,
                exclude,
                skip_larger_than,
                add_timestamp,
            )
            .await?;

            info!("Folder backup completed successfully:");
            info!("  Files processed: {}", stats.files_processed);
            info!("  Files skipped: {}", stats.files_skipped);
            info!("  Files failed: {}", stats.files_failed);
            info!("  Total bytes transferred: {}", stats.total_bytes);
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
            // Determine which backup to restore
            let backup_key = if latest {
                info!(
                    "Looking for latest PostgreSQL backup for database: {}",
                    database
                );

                let latest_backup =
                    get_latest_backup(&s3_client, &cli.bucket, &cli.prefix, "postgres", &database)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("No backups found for database {}", database)
                        })?;

                info!(
                    "Found latest backup from {}: {}",
                    latest_backup.last_modified, latest_backup.key
                );

                latest_backup.key
            } else if let Some(s3_key) = key {
                info!("Using specified backup key: {}", s3_key);
                s3_key
            } else {
                return Err(anyhow::anyhow!(
                    "Either --key or --latest must be specified"
                ));
            };

            // Download the backup
            let backup_data = download_backup(&s3_client, &cli.bucket, &backup_key).await?;

            // Restore the database
            restore_postgres(
                &database,
                &host,
                port,
                &username,
                password.as_deref(),
                backup_data,
                force_docker,
                drop_db,
            )
            .await?;

            info!("PostgreSQL database restore completed successfully");
        }
        Commands::RestoreMysql {
            database,
            host,
            port,
            username,
            password,
            key,
            latest,
            drop_db,
        } => {
            // Determine which backup to restore
            // let backup_key = if latest {
            //     info!("Looking for latest MySQL backup for database: {}", database);

            //     let latest_backup =
            //         get_latest_backup(&s3_client, &cli.bucket, &cli.prefix, "mysql", &database)
            //             .await?
            //             .ok_or_else(|| {
            //                 anyhow::anyhow!("No backups found for database {}", database)
            //             })?;

            //     info!(
            //         "Found latest backup from {}: {}",
            //         latest_backup.last_modified, latest_backup.key
            //     );

            //     latest_backup.key
            // } else if let Some(s3_key) = key {
            //     info!("Using specified backup key: {}", s3_key);
            //     s3_key
            // } else {
            //     return Err(anyhow::anyhow!(
            //         "Either --key or --latest must be specified"
            //     ));
            // };

            // // Download the backup
            // let backup_data = download_backup(&s3_client, &cli.bucket, &backup_key).await?;

            // // Restore the database
            // restore_mysql(
            //     &database,
            //     &host,
            //     port,
            //     &username,
            //     password.as_deref(),
            //     backup_data,
            //     drop_db,
            // )
            // .await?;

            // info!("MySQL database restore completed successfully");
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
